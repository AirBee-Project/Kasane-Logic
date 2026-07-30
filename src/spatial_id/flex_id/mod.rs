use alloc::vec::Vec;

pub mod constructor;
pub mod convert;
pub mod encode;
pub mod impls;
pub mod ops;

use crate::{
    Error, Interval, Side, SpatialIdError,
    spatial_id::{
        range_id::convert::{split_f, split_xy},
        time_cells,
        zoom_level::{TZoomLevel, ZoomLevel},
    },
};

/// 拡張時空間ID。FlexTree のノードアドレスでもある。
///
/// **4軸（F / X / Y / T）とも「ズームレベル＋インデックス」で対称に持つ**のが
/// [`SingleId`](crate::SingleId) / [`RangeId`](crate::RangeId) との違いで、軸ごとに
/// 粒度を変えられる。時間軸のズームは空間軸（最大30）と範囲が違う
/// （[`TZoomLevel`]、最大35＝1秒）ので型で区別する。
#[derive(Clone, Copy, PartialEq, Debug, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct FlexId {
    f_zoomlevel: ZoomLevel,
    f_index: i32,
    x_zoomlevel: ZoomLevel,
    x_index: u32,
    y_zoomlevel: ZoomLevel,
    y_index: u32,
    t_zoomlevel: TZoomLevel,
    t_index: u64,
}

impl FlexId {
    pub const UPPER_MAX: FlexId = FlexId {
        f_zoomlevel: ZoomLevel::MIN,
        f_index: 0,
        x_zoomlevel: ZoomLevel::MIN,
        x_index: 0,
        y_zoomlevel: ZoomLevel::MIN,
        y_index: 0,
        t_zoomlevel: TZoomLevel::MIN,
        t_index: 0,
    };

    pub const LOWER_MAX: FlexId = FlexId {
        f_zoomlevel: ZoomLevel::MIN,
        f_index: -1,
        x_zoomlevel: ZoomLevel::MIN,
        x_index: 0,
        y_zoomlevel: ZoomLevel::MIN,
        y_index: 0,
        t_zoomlevel: TZoomLevel::MIN,
        t_index: 0,
    };

    pub fn f_zoomlevel(&self) -> u8 {
        self.f_zoomlevel.get()
    }
    pub fn x_zoomlevel(&self) -> u8 {
        self.x_zoomlevel.get()
    }
    pub fn y_zoomlevel(&self) -> u8 {
        self.y_zoomlevel.get()
    }
    /// 時間軸のズームレベル。1セルは `2^(35 - t_zoomlevel)` 秒。
    pub fn t_zoomlevel(&self) -> u8 {
        self.t_zoomlevel.get()
    }
    pub fn f_index(&self) -> i32 {
        self.f_index
    }
    pub fn x_index(&self) -> u32 {
        self.x_index
    }
    pub fn y_index(&self) -> u32 {
        self.y_index
    }
    /// 時間軸のインデックス。[`t`](Self::t) と同じ値だが、`t_zoomlevel()` と対で読む用。
    pub fn t_index(&self) -> u64 {
        self.t_index
    }

    /// この [`FlexId`] が占める時間セルの単位（秒幅 `2^(35 - t_zoomlevel)`）を返す。
    ///
    /// [`FlexId`] のセルは必ず2の冪秒なので、この幅がそのまま「その区間を表せる最も粗い単位」
    /// でもある（開始秒は幅の倍数なので `gcd(start, width) == width`）。
    ///
    /// ```
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::{Interval, SingleId};
    /// // 1時間ぶんは複数の2進セルへ分解されるが、各セルの秒幅は必ず2の冪になる。
    /// let id = SingleId::new(3, 0, 1, 1).unwrap().with_time(Interval::HOUR, 2).unwrap();
    /// for cell in id.into_iter() {
    ///     assert!(cell.interval().seconds().is_power_of_two());
    /// }
    /// # }
    /// ```
    pub fn interval(&self) -> Interval {
        let (start, end) = self.seconds_range();
        Interval::from_seconds_unchecked(end - start)
    }

    /// この [`FlexId`] の時間インデックス。[`interval`](Self::interval) を単位とする。
    pub fn t(&self) -> u64 {
        self.t_index
    }

    /// この [`FlexId`] が占める絶対秒区間 `[start, end)` を返す。
    pub fn seconds_range(&self) -> (u64, u64) {
        time_cells::cell_seconds_range(self.t_zoomlevel.get(), self.t_index)
    }

    /// 時間セルを差し替えた自身を返す。クレート内部専用（検証済みの値を渡すこと）。
    pub(crate) fn with_time_cell(mut self, zoom: u8, index: u64) -> Self {
        debug_assert!(TZoomLevel::new(zoom).is_ok_and(|z| z.check_index(index).is_ok()));
        self.t_zoomlevel = unsafe { TZoomLevel::new_unchecked(zoom) };
        self.t_index = index;
        self
    }

    /// 指定したズームレベルでの空間的な親ID（包含する最小のFlexId）を返す。
    /// 指定されたズームが現在のズーム以上の場合は、各軸についてそのままのズームとインデックスを返す。
    pub fn spatial_parent_at_zoom(&self, target_z: u8) -> Result<FlexId, Error> {
        let f_zoomlevel = self.f_zoomlevel().min(target_z);
        let x_zoomlevel = self.x_zoomlevel().min(target_z);
        let y_zoomlevel = self.y_zoomlevel().min(target_z);

        let shift_f = self.f_zoomlevel() - f_zoomlevel;
        let shift_x = self.x_zoomlevel() - x_zoomlevel;
        let shift_y = self.y_zoomlevel() - y_zoomlevel;

        let f_index = self.f_index() >> shift_f;
        let x_index = self.x_index() >> shift_x;
        let y_index = self.y_index() >> shift_y;

        FlexId::new(
            f_zoomlevel,
            f_index,
            x_zoomlevel,
            x_index,
            y_zoomlevel,
            y_index,
        )
        .map(|id| id.with_time_cell(self.t_zoomlevel.get(), self.t_index))
    }

    /// このFlexIdを高さ（F）方向へ、ズーム `z` のセル `index` 個分だけ引き延ばした結果を返す。
    ///
    /// [`shift_f`](Self::shift_f) がセルを移動するのに対し、こちらは元のセルを残したまま
    /// 指定方向（`index` の符号）へセルを継ぎ足して占有区間を拡張する。`index == 0` なら
    /// 元のセルと等価。占有区間は整列したセル群へ分解されるため複数の [`FlexId`] を返す。
    /// XY方向の値は変更しない。
    ///
    /// # バリデーション
    /// - `z` が [`ZoomLevel::MAX`] を超える場合は [`SpatialIdError::ZOutOfRange`] を返す。
    /// - `index` がズーム `z` のF範囲外の場合は [`SpatialIdError::FOutOfRange`] を返す。
    /// - 拡張後の区間が `max(f_zoomlevel, z)` のF範囲を超える場合は
    ///   [`SpatialIdError::FOutOfRange`] を返す。
    pub fn stretch_f(&self, z: u8, index: i32) -> Result<impl Iterator<Item = FlexId>, Error> {
        if z > ZoomLevel::MAX.get() {
            return Err(SpatialIdError::ZOutOfRange { z }.into());
        }
        if index < ZoomLevel::new(z).unwrap().f_min() || index > ZoomLevel::new(z).unwrap().f_max()
        {
            return Err(SpatialIdError::FOutOfRange { z, f: index }.into());
        }

        let f_zoomlevel = self.f_zoomlevel();
        let max_z = f_zoomlevel.max(z);

        let cell_scale = 1_i32 << (max_z - f_zoomlevel);
        let delta = index * (1_i32 << (max_z - z));

        // 元セルの占有区間 [base_left, base_right] を、符号に応じて片側だけ拡張する。
        let base_left = self.f_index() * cell_scale;
        let base_right = base_left + cell_scale - 1;
        let (left, right) = if delta >= 0 {
            (base_left, base_right + delta)
        } else {
            (base_left + delta, base_right)
        };

        if left < ZoomLevel::new(max_z)?.f_min() {
            return Err(SpatialIdError::FOutOfRange { z: max_z, f: left }.into());
        }
        if right > ZoomLevel::new(max_z)?.f_max() {
            return Err(SpatialIdError::FOutOfRange { z: max_z, f: right }.into());
        }

        let x_zoomlevel = self.x_zoomlevel();
        let x_index = self.x_index();
        let y_zoomlevel = self.y_zoomlevel();
        let y_index = self.y_index();
        let (t_zoomlevel, t_index) = (self.t_zoomlevel.get(), self.t_index);

        Ok(
            split_f(max_z, [left, right]).map(move |(seg_z, seg_index)| {
                unsafe {
                    FlexId::new_unchecked(
                        seg_z,
                        seg_index,
                        x_zoomlevel,
                        x_index,
                        y_zoomlevel,
                        y_index,
                    )
                }
                .with_time_cell(t_zoomlevel, t_index)
            }),
        )
    }

    /// このFlexIdを東西（X）方向へ、ズーム `z` のセル `index` 個分だけ引き延ばした結果を返す。
    ///
    /// 元のセルを残したまま指定方向（`index` の符号）へ拡張する。X方向は東西に巡回するため、
    /// 拡張量が大きいと境界をまたいで分割され、`max(x_zoomlevel, z)` の周長以上では全周を覆う。
    /// F・Y方向の値は変更しない。
    ///
    /// # バリデーション
    /// - `z` が [`ZoomLevel::MAX`] を超える場合は [`SpatialIdError::ZOutOfRange`] を返す。
    pub fn stretch_x(&self, z: u8, index: i32) -> Result<impl Iterator<Item = FlexId>, Error> {
        if z > ZoomLevel::MAX.get() {
            return Err(SpatialIdError::ZOutOfRange { z }.into());
        }

        let x_zoomlevel = self.x_zoomlevel();
        let max_z = x_zoomlevel.max(z);

        let circumference = 1_i64 << max_z;
        let cell_scale = 1_i64 << (max_z - x_zoomlevel);
        let delta = index as i64 * (1_i64 << (max_z - z));

        let base_left = self.x_index() as i64 * cell_scale;
        let base_right = base_left + cell_scale - 1;
        let (left, right) = if delta >= 0 {
            (base_left, base_right + delta)
        } else {
            (base_left + delta, base_right)
        };

        // 占有幅が周長以上なら全周。そうでなければ巡回させ、境界跨ぎは2区間に分ける。
        let ranges: Vec<[u32; 2]> = if right - left + 1 >= circumference {
            vec![[0, (circumference - 1) as u32]]
        } else {
            let left_wrapped = left.rem_euclid(circumference);
            let right_wrapped = right.rem_euclid(circumference);
            if left_wrapped <= right_wrapped {
                vec![[left_wrapped as u32, right_wrapped as u32]]
            } else {
                vec![
                    [left_wrapped as u32, (circumference - 1) as u32],
                    [0, right_wrapped as u32],
                ]
            }
        };

        let f_zoomlevel = self.f_zoomlevel();
        let f_index = self.f_index();
        let y_zoomlevel = self.y_zoomlevel();
        let y_index = self.y_index();
        let (t_zoomlevel, t_index) = (self.t_zoomlevel.get(), self.t_index);

        Ok(ranges
            .into_iter()
            .flat_map(move |range| split_xy(max_z, range))
            .map(move |(seg_z, seg_index)| {
                unsafe {
                    FlexId::new_unchecked(
                        f_zoomlevel,
                        f_index,
                        seg_z,
                        seg_index,
                        y_zoomlevel,
                        y_index,
                    )
                }
                .with_time_cell(t_zoomlevel, t_index)
            }))
    }

    /// このFlexIdを南北（Y）方向へ、ズーム `z` のセル `index` 個分だけ引き延ばした結果を返す。
    ///
    /// 元のセルを残したまま指定方向（`index` の符号）へ拡張する。Y方向は巡回せず
    /// `[0[z]]` に制限される。F・X方向の値は変更しない。
    ///
    /// # バリデーション
    /// - `z` が [`ZoomLevel::MAX`] を超える場合は [`SpatialIdError::ZOutOfRange`] を返す。
    /// - 拡張後の区間が `max(y_zoomlevel, z)` のY範囲を超える場合は
    ///   [`SpatialIdError::YOutOfRange`] を返す。
    pub fn stretch_y(&self, z: u8, index: i32) -> Result<impl Iterator<Item = FlexId>, Error> {
        if z > ZoomLevel::MAX.get() {
            return Err(SpatialIdError::ZOutOfRange { z }.into());
        }

        let y_zoomlevel = self.y_zoomlevel();
        let max_z = y_zoomlevel.max(z);

        let cell_scale = 1_i64 << (max_z - y_zoomlevel);
        let delta = index as i64 * (1_i64 << (max_z - z));

        let base_left = self.y_index() as i64 * cell_scale;
        let base_right = base_left + cell_scale - 1;
        let (left, right) = if delta >= 0 {
            (base_left, base_right + delta)
        } else {
            (base_left + delta, base_right)
        };

        let y_max = ZoomLevel::new(max_z)?.xy_max() as i64;
        if left < 0 || right > y_max {
            let offending = if left < 0 { left } else { right };
            return Err(SpatialIdError::YOutOfRange {
                z: max_z,
                y: offending.clamp(0, u32::MAX as i64) as u32,
            }
            .into());
        }

        let f_zoomlevel = self.f_zoomlevel();
        let f_index = self.f_index();
        let x_zoomlevel = self.x_zoomlevel();
        let x_index = self.x_index();
        let (t_zoomlevel, t_index) = (self.t_zoomlevel.get(), self.t_index);

        Ok(
            split_xy(max_z, [left as u32, right as u32]).map(move |(seg_z, seg_index)| {
                unsafe {
                    FlexId::new_unchecked(
                        f_zoomlevel,
                        f_index,
                        x_zoomlevel,
                        x_index,
                        seg_z,
                        seg_index,
                    )
                }
                .with_time_cell(t_zoomlevel, t_index)
            }),
        )
    }

    /// F方向で二つに切り分ける。[`ZoomLevel::MAX`] なら [`None`]。
    ///
    /// 分割後も各軸が有効範囲に収まることは構築時に保証される（F は
    /// `[-2^z, 2^z-1]` を2倍して `side` を足すと過不足なく `z+1` の範囲になり、
    /// X/Y は値を変えずに引き継ぐ）ため、[`FlexId::new`] の再検証を通さない。
    pub fn split_f(&self, side: Side) -> Option<FlexId> {
        let f_zoomlevel = self.f_zoomlevel.deeper()?;
        let f_index = self.f_index * 2 + side as i32;
        debug_assert!(f_zoomlevel.check_f(f_index).is_ok(), "分割後のFが範囲外");

        Some(FlexId {
            f_zoomlevel,
            f_index,
            x_zoomlevel: self.x_zoomlevel,
            x_index: self.x_index,
            y_zoomlevel: self.y_zoomlevel,
            y_index: self.y_index,
            t_zoomlevel: self.t_zoomlevel,
            t_index: self.t_index,
        })
    }

    /// X方向で二つに切り分ける。[`ZoomLevel::MAX`] なら [`None`]。
    ///
    /// 再検証を省ける理由は [`split_f`](Self::split_f) と同じ。
    pub fn split_x(&self, side: Side) -> Option<FlexId> {
        let x_zoomlevel = self.x_zoomlevel.deeper()?;
        let x_index = self.x_index * 2 + side as u32;
        debug_assert!(x_zoomlevel.check_x(x_index).is_ok(), "分割後のXが範囲外");

        Some(FlexId {
            f_zoomlevel: self.f_zoomlevel,
            f_index: self.f_index,
            x_zoomlevel,
            x_index,
            y_zoomlevel: self.y_zoomlevel,
            y_index: self.y_index,
            t_zoomlevel: self.t_zoomlevel,
            t_index: self.t_index,
        })
    }

    /// Y方向で二つに切り分ける。[`ZoomLevel::MAX`] なら [`None`]。
    ///
    /// 再検証を省ける理由は [`split_f`](Self::split_f) と同じ。
    pub fn split_y(&self, side: Side) -> Option<FlexId> {
        let y_zoomlevel = self.y_zoomlevel.deeper()?;
        let y_index = self.y_index * 2 + side as u32;
        debug_assert!(y_zoomlevel.check_y(y_index).is_ok(), "分割後のYが範囲外");

        Some(FlexId {
            f_zoomlevel: self.f_zoomlevel,
            f_index: self.f_index,
            x_zoomlevel: self.x_zoomlevel,
            x_index: self.x_index,
            y_zoomlevel,
            y_index,
            t_zoomlevel: self.t_zoomlevel,
            t_index: self.t_index,
        })
    }

    /// 時間軸で二つに切り分ける。[`TZoomLevel::MAX`] なら [`None`]。
    ///
    /// 再検証を省ける理由は [`split_f`](Self::split_f) と同じ
    /// （インデックスを2倍して `side` を足すと過不足なく1段深い範囲になる）。
    pub fn split_t(&self, side: Side) -> Option<FlexId> {
        let t_zoomlevel = self.t_zoomlevel.deeper()?;
        let t_index = self.t_index * 2 + side as u64;
        debug_assert!(
            t_zoomlevel.check_index(t_index).is_ok(),
            "分割後のTが範囲外"
        );

        Some(FlexId {
            f_zoomlevel: self.f_zoomlevel,
            f_index: self.f_index,
            x_zoomlevel: self.x_zoomlevel,
            x_index: self.x_index,
            y_zoomlevel: self.y_zoomlevel,
            y_index: self.y_index,
            t_zoomlevel,
            t_index,
        })
    }

    /// この [`FlexId`] が `other` と **面を共有** しているかを判定します。X 軸は循環（対蹠経度で東西端が接続）を考慮します。辺・頂点だけで接する場合、領域が重なる場合、離れている場合はいずれも `false` を返します。判定は空間 3 軸（F / X / Y）のみで行い、時間 ID は考慮しません。
    ///
    /// ```
    /// # use kasane_logic::FlexId;
    /// let a = FlexId::new(4, 5, 4, 5, 4, 5).unwrap();
    /// let east = FlexId::new(4, 5, 4, 6, 4, 5).unwrap(); // X+1（面で接する）
    /// let diag = FlexId::new(4, 5, 4, 6, 4, 6).unwrap(); // X+1,Y+1（辺で接する）
    /// assert!(a.shares_face(&east));
    /// assert!(!a.shares_face(&diag));
    /// assert!(!a.shares_face(&a)); // 重なり（自身）は面共有ではない
    /// ```
    pub fn shares_face(&self, other: &FlexId) -> bool {
        #[derive(PartialEq)]
        enum Rel {
            Overlap,
            Adjacent,
            Separated,
        }

        fn axis_range(zoom: u8, index: i64, common: u8) -> (i64, i64) {
            let shift = (common - zoom) as i64;
            (index << shift, ((index + 1) << shift) - 1)
        }

        fn classify(a: (i64, i64), b: (i64, i64), cyclic_width: Option<i64>) -> Rel {
            if a.0.max(b.0) <= a.1.min(b.1) {
                return Rel::Overlap;
            }
            let mut adjacent = a.1 + 1 == b.0 || b.1 + 1 == a.0;
            if let Some(w) = cyclic_width
                && ((a.1 + 1).rem_euclid(w) == b.0.rem_euclid(w)
                    || (b.1 + 1).rem_euclid(w) == a.0.rem_euclid(w))
            {
                adjacent = true;
            }
            if adjacent {
                Rel::Adjacent
            } else {
                Rel::Separated
            }
        }

        let cf = self.f_zoomlevel().max(other.f_zoomlevel());
        let rf = classify(
            axis_range(self.f_zoomlevel(), self.f_index() as i64, cf),
            axis_range(other.f_zoomlevel(), other.f_index() as i64, cf),
            None,
        );
        let cx = self.x_zoomlevel().max(other.x_zoomlevel());
        let rx = classify(
            axis_range(self.x_zoomlevel(), self.x_index() as i64, cx),
            axis_range(other.x_zoomlevel(), other.x_index() as i64, cx),
            Some(1i64 << cx),
        );
        let cy = self.y_zoomlevel().max(other.y_zoomlevel());
        let ry = classify(
            axis_range(self.y_zoomlevel(), self.y_index() as i64, cy),
            axis_range(other.y_zoomlevel(), other.y_index() as i64, cy),
            None,
        );

        let rels = [rf, rx, ry];
        rels.iter().filter(|r| **r == Rel::Adjacent).count() == 1
            && rels.iter().filter(|r| **r == Rel::Overlap).count() == 2
    }

    /// この [`FlexId`] を、指定した各軸のズームレベルで区切られたシャード単位で分割し、親と「シャード内に含まれる対象の分割部分」のペアを列挙する。
    ///
    /// 戻り値のイテレータが生成する要素は `(親, 分割部分)` 。
    pub fn shard(
        &self,
        f_zoomlevel: ZoomLevel,
        x_zoomlevel: ZoomLevel,
        y_zoomlevel: ZoomLevel,
    ) -> impl Iterator<Item = (FlexId, FlexId)> {
        let sz_f = self.f_zoomlevel();
        let tz_f = f_zoomlevel.get();
        let (f_start, f_end) = if tz_f <= sz_f {
            let shift = sz_f - tz_f;
            let idx = self.f_index() >> shift;
            (idx, idx)
        } else {
            let shift = tz_f - sz_f;
            let si = self.f_index() as i64;
            ((si << shift) as i32, (((si + 1) << shift) - 1) as i32)
        };

        let sz_x = self.x_zoomlevel();
        let tz_x = x_zoomlevel.get();
        let (x_start, x_end) = if tz_x <= sz_x {
            let shift = sz_x - tz_x;
            let idx = self.x_index() >> shift;
            (idx, idx)
        } else {
            let shift = tz_x - sz_x;
            let si = self.x_index() as u64;
            ((si << shift) as u32, (((si + 1) << shift) - 1) as u32)
        };

        let sz_y = self.y_zoomlevel();
        let tz_y = y_zoomlevel.get();
        let (y_start, y_end) = if tz_y <= sz_y {
            let shift = sz_y - tz_y;
            let idx = self.y_index() >> shift;
            (idx, idx)
        } else {
            let shift = tz_y - sz_y;
            let si = self.y_index() as u64;
            ((si << shift) as u32, (((si + 1) << shift) - 1) as u32)
        };

        let seg_fz = sz_f.max(tz_f);
        let seg_xz = sz_x.max(tz_x);
        let seg_yz = sz_y.max(tz_y);

        let self_fi = self.f_index();
        let self_xi = self.x_index();
        let self_yi = self.y_index();

        let (tz, ti) = (self.t_zoomlevel.get(), self.t_index);

        (f_start..=f_end).flat_map(move |f_idx| {
            (x_start..=x_end).flat_map(move |x_idx| {
                (y_start..=y_end).map(move |y_idx| {
                    let seg_fi = if sz_f >= tz_f { self_fi } else { f_idx };
                    let seg_xi = if sz_x >= tz_x { self_xi } else { x_idx };
                    let seg_yi = if sz_y >= tz_y { self_yi } else { y_idx };

                    let parent =
                        unsafe { FlexId::new_unchecked(tz_f, f_idx, tz_x, x_idx, tz_y, y_idx) }
                            .with_time_cell(tz, ti);
                    let seg = unsafe {
                        FlexId::new_unchecked(seg_fz, seg_fi, seg_xz, seg_xi, seg_yz, seg_yi)
                    }
                    .with_time_cell(tz, ti);

                    (parent, seg)
                })
            })
        })
    }
}
