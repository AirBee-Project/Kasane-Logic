pub mod constructor;
pub mod convert;
pub mod encode;
pub mod impls;
pub mod ops;
pub mod random;
pub mod test;

use crate::{
    SpatialId, SpatialIdError, TemporalId,
    error::Error,
    spatial_id::constants::{F_MAX, F_MIN, MAX_ZOOM_LEVEL, XY_MAX},
};

/// SingleIdは標準的な時空間 ID を表す型。
///
/// 内部的には下記のような構造体で構成されている。
///
/// この型は `PartialOrd` / `Ord` を実装していますが、これは主に`BTreeSet` や `BTreeMap` などの順序付きコレクションでの格納・探索用であり、実際の空間的な「大小」を意味するものではない。
///
/// ```
/// # use kasane_logic::TemporalId;
/// pub struct SingleId {
///     z: u8,
///     f: i32,
///     x: u32,
///     y: u32,
//
///     temporal_id: TemporalId,
/// }
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct SingleId {
    z: u8,
    f: i32,
    x: u32,
    y: u32,
    temporal_id: TemporalId,
}

impl SingleId {
    /// この `SingleId` が保持しているズームレベル `z` を返します。
    ///
    /// ```no_run
    /// # use kasane_logic::{Error, SingleId, SpatialIdError};
    /// let id = SingleId::new(5, 3, 2, 10).unwrap();
    /// assert_eq!(id.z(), 5u8);
    /// ```
    pub fn z(&self) -> u8 {
        self.z
    }

    /// この `SingleId` が保持している F インデックス `f` を返します。
    ///
    /// ```
    /// # use kasane_logic::SingleId;
    /// let id = SingleId::new(5, 3, 2, 10).unwrap();
    /// assert_eq!(id.f(), 3i32);
    /// ```
    pub fn f(&self) -> i32 {
        self.f
    }

    /// この `SingleId` が保持している X インデックス `x` を返します。
    ///
    /// ```
    /// # use kasane_logic::SingleId;
    /// let id = SingleId::new(5, 3, 2, 10).unwrap();
    /// assert_eq!(id.x(), 2u32);
    /// ```
    pub fn x(&self) -> u32 {
        self.x
    }

    /// この `SingleId` が保持している Y インデックス `y` を返します。
    ///
    /// ```
    /// # use kasane_logic::SingleId;
    /// let id = SingleId::new(5, 3, 2, 10).unwrap();
    /// assert_eq!(id.y(), 10u32);
    /// ```
    pub fn y(&self) -> u32 {
        self.y
    }

    /// F インデックスを更新します。
    ///
    /// 与えられた `value` が、現在のズームレベル `z` に対応する
    /// `F_MIN[z]..=F_MAX[z]` の範囲内にあるかを検証し、範囲外の場合は [`Error`] を返します。
    ///
    /// # パラメータ
    /// * `value` — 新しい F インデックス
    ///
    /// # バリデーション
    /// - `value` が許容範囲外の場合、[`SpatialIdError::FOutOfRange`] を返します。
    ///
    /// 正常な更新:
    /// ```
    /// # use kasane_logic::SingleId;
    /// let mut id = SingleId::new(5, 3, 2, 10).unwrap();
    /// id.set_f(4).unwrap();
    /// assert_eq!(id.f(), 4);
    /// ```
    ///
    /// 範囲外の検知:
    /// ```
    /// # use kasane_logic::{Error, SingleId, SpatialIdError};
    /// let mut id = SingleId::new(3, 3, 2, 7).unwrap();
    /// let result = id.set_f(999);
    /// assert!(matches!(result, Err(Error::SpatialId(SpatialIdError::FOutOfRange { z: 3, f: 999 }))));
    /// ```
    pub fn set_f(&mut self, value: i32) -> Result<(), Error> {
        let min = F_MIN[self.z() as usize];
        let max = F_MAX[self.z() as usize];
        if value < min || value > max {
            return Err(SpatialIdError::FOutOfRange {
                f: value,
                z: self.z,
            }
            .into());
        }
        self.f = value;
        Ok(())
    }

    /// X インデックスを更新します。
    ///
    /// 与えられた `value` が、現在のズームレベル `z` に対応する
    /// `0..=XY_MAX[z]` の範囲内にあるかを検証し、範囲外の場合は [`Error`] を返します。
    ///
    /// # パラメータ
    /// * `value` — 新しい X インデックス
    ///
    /// # バリデーション
    /// - `value` が許容範囲外の場合、[`SpatialIdError::XOutOfRange`] を返します。
    ///
    /// 正常な更新:
    /// ```
    /// # use kasane_logic::SingleId;
    /// let mut id = SingleId::new(5, 3, 2, 10).unwrap();
    /// id.set_x(4).unwrap();
    /// assert_eq!(id.x(), 4);
    /// ```
    ///
    /// 範囲外の検知
    /// ```
    /// # use kasane_logic::{Error, SingleId, SpatialIdError};
    /// let mut id = SingleId::new(3, 3, 2, 7).unwrap();
    /// let result = id.set_x(999);
    /// assert!(matches!(result, Err(Error::SpatialId(SpatialIdError::XOutOfRange { z: 3, x: 999 }))));
    /// ```
    pub fn set_x(&mut self, value: u32) -> Result<(), Error> {
        let max = XY_MAX[self.z() as usize];
        if value > max {
            return Err(SpatialIdError::XOutOfRange {
                x: value,
                z: self.z,
            }
            .into());
        }
        self.x = value;
        Ok(())
    }

    /// Y インデックスを更新します。
    ///
    /// 与えられた `value` が、現在のズームレベル `z` に対応する
    /// `0..=XY_MAX[z]` の範囲内にあるかを検証し、範囲外の場合は [`Error`] を返します。
    ///
    /// # パラメータ
    /// * `value` — 新しい Y インデックス
    ///
    /// # バリデーション
    /// - `value` が許容範囲外の場合、[`SpatialIdError::YOutOfRange`] を返します。
    ///
    /// 正常な更新
    /// ```
    /// # use kasane_logic::SingleId;
    /// let mut id = SingleId::new(5, 3, 2, 10).unwrap();
    /// id.set_y(8).unwrap();
    /// assert_eq!(id.y(), 8);
    /// ```
    ///
    /// 範囲外の検知
    /// ```
    /// # use kasane_logic::{Error, SingleId, SpatialIdError};
    /// let mut id = SingleId::new(3, 3, 2, 7).unwrap();
    /// let result = id.set_y(999);
    /// assert!(matches!(result, Err(Error::SpatialId(SpatialIdError::YOutOfRange { z: 3, y: 999 }))));
    /// ```
    pub fn set_y(&mut self, value: u32) -> Result<(), Error> {
        let max = XY_MAX[self.z() as usize];
        if value > max {
            return Err(SpatialIdError::YOutOfRange {
                y: value,
                z: self.z,
            }
            .into());
        }
        self.y = value;
        Ok(())
    }

    /// 指定したズームレベル `target_z` に細分化した、この `SingleId` を含むすべての子 `SingleId` を生成します。
    ///
    /// # パラメータ
    /// * `target_z` — 生成したい子 `SingleId` のズームレベル
    ///
    /// # バリデーション
    /// - `target_z` が現在のズームレベルより浅い場合は、[`SpatialIdError::ZoomLevelTransitionOutOfRange`] を返します。
    /// - `target_z` が本クレートで扱える最大ズームレベルを超える場合は、[`SpatialIdError::ZOutOfRange`] を返します。
    ///
    /// 1段深いズームへの細分化
    /// ```
    /// # use kasane_logic::SingleId;
    /// let id = SingleId::new(3, 3, 2, 7).unwrap();
    ///
    /// // target_z = 4 のため F, X, Y はそれぞれ 2 分割される
    /// let children: Vec<_> = id.spatial_children_at_zoom(4).unwrap().collect();
    ///
    /// assert_eq!(children.len(), 8); // 2 × 2 × 2
    ///
    /// // 最初の要素を確認（f, x, y の下限側）
    /// let first = &children[0];
    /// assert_eq!(first.z(), 4);
    /// assert_eq!(first.f(), 3 * 2);   // 2
    /// assert_eq!(first.x(), 2 * 2);   // 6
    /// assert_eq!(first.y(), 7 * 2);   // 8
    /// ```
    ///
    /// 現在より浅いズームを指定した場合
    /// ```
    /// # use kasane_logic::{Error, SingleId, SpatialIdError};
    /// let id = SingleId::new(3, 3, 2, 7).unwrap();
    /// let result = id.spatial_children_at_zoom(2);
    /// assert!(matches!(result, Err(Error::SpatialId(SpatialIdError::ZoomLevelTransitionOutOfRange { current_z: 3, target_z: 2 }))));
    /// ```
    pub fn spatial_children_at_zoom(
        &self,
        target_z: u8,
    ) -> Result<impl Iterator<Item = SingleId>, Error> {
        if target_z < self.z {
            return Err(SpatialIdError::ZoomLevelTransitionOutOfRange {
                current_z: self.z,
                target_z,
            }
            .into());
        }

        if target_z as usize > MAX_ZOOM_LEVEL {
            return Err(SpatialIdError::ZOutOfRange { z: target_z }.into());
        }

        let difference = target_z - self.z;

        let scale_f = 2_i32.pow(difference as u32);
        let scale_xy = 2_u32.pow(difference as u32);

        let f_start = self.f * scale_f;
        let x_start = self.x * scale_xy;
        let y_start = self.y * scale_xy;

        let f_range = f_start..=f_start + scale_f - 1;
        let x_range = x_start..=x_start + scale_xy - 1;
        let y_range = y_start..=y_start + scale_xy - 1;

        Ok(f_range.flat_map(move |f| {
            let x_range = x_range.clone();
            let y_range = y_range.clone();

            x_range.flat_map(move |x| {
                y_range.clone().map(move |y| SingleId {
                    z: target_z,
                    f,
                    x,
                    y,

                    temporal_id: self.temporal().clone(),
                })
            })
        }))
    }

    /// 指定したズームレベル `target_z` に縮約した、この `SingleId` の親 `SingleId` を返します。
    ///
    /// # パラメータ
    /// * `target_z` — 取得したい親 `SingleId` のズームレベル
    ///
    /// # バリデーション
    /// - `target_z` が現在のズームレベルより深い場合は、[`SpatialIdError::ZoomLevelTransitionOutOfRange`] を返します。
    /// - `target_z` が本クレートで扱える最大ズームレベルを超える場合は、[`SpatialIdError::ZOutOfRange`] を返します。
    ///
    /// 1段浅いズームへの縮約
    /// ```
    /// # use kasane_logic::SingleId;
    /// let id = SingleId::new(4, 6, 9, 14).unwrap();
    ///
    /// let parent = id.spatial_parent_at_zoom(3).unwrap();
    ///
    /// assert_eq!(parent.z(), 3u8);
    /// assert_eq!(parent.f(), 3i32);
    /// assert_eq!(parent.x(), 4u32);
    /// assert_eq!(parent.y(), 7u32);
    /// ```
    ///
    /// Fが負の場合の挙動
    /// ```
    /// # use kasane_logic::SingleId;
    /// let id = SingleId::new(4, -1, 8, 12).unwrap();
    ///
    /// let parent = id.spatial_parent_at_zoom(3).unwrap();
    ///
    /// assert_eq!(parent.z(), 3u8);
    /// assert_eq!(parent.f(), -1i32);
    /// assert_eq!(parent.x(), 4u32);
    /// assert_eq!(parent.y(), 6u32);
    /// ```
    ///
    /// 現在より深いズームを指定した場合:
    /// ```
    /// # use kasane_logic::{Error, SingleId, SpatialIdError};
    /// let id = SingleId::new(3, 3, 2, 7).unwrap();
    /// let result = id.spatial_parent_at_zoom(4);
    /// assert!(matches!(result, Err(Error::SpatialId(SpatialIdError::ZoomLevelTransitionOutOfRange { current_z: 3, target_z: 4 }))));
    /// ```
    pub fn spatial_parent_at_zoom(&self, target_z: u8) -> Result<SingleId, Error> {
        if target_z > self.z {
            return Err(SpatialIdError::ZoomLevelTransitionOutOfRange {
                current_z: self.z,
                target_z,
            }
            .into());
        }

        if target_z as usize > MAX_ZOOM_LEVEL {
            return Err(SpatialIdError::ZOutOfRange { z: target_z }.into());
        }

        let difference = self.z - target_z;
        let f = if self.f == -1 {
            -1
        } else {
            self.f >> difference
        };
        let x = self.x >> (difference as u32);
        let y = self.y >> (difference as u32);

        Ok(SingleId {
            z: target_z,
            f,
            x,
            y,

            temporal_id: self.temporal().clone(),
        })
    }

    /// この [`SingleId`] と同じ親を共有する兄弟 [`SingleId`] を 7 個返す。
    ///
    /// 返される 7 個と自分自身を合わせると、1つ上のズームレベルにある親 [`SingleId`] を構成する。
    /// `z = 0` の場合は親を持たないため `None` を返す。
    ///
    /// # 動作例
    ///
    /// 兄弟の列挙:
    /// ```
    /// # use kasane_logic::SingleId;
    /// let id = SingleId::new(3, 3, 2, 7).unwrap();
    /// let siblings = id.spatial_siblings().unwrap();
    ///
    /// assert_eq!(siblings.len(), 7);
    /// assert!(siblings.iter().all(|s| s.spatial_parent_at_zoom(2).unwrap() == id.spatial_parent_at_zoom(2).unwrap()));
    /// assert!(!siblings.contains(&id));
    /// ```
    ///
    /// 最上位の ID の場合:
    /// ```
    /// # use kasane_logic::SingleId;
    /// let id = SingleId::new(0, 0, 0, 0).unwrap();
    /// assert!(id.spatial_siblings().is_none());
    /// ```
    pub fn spatial_siblings(&self) -> Option<[SingleId; 7]> {
        if self.z == 0 {
            return None;
        }

        let parent_z = self.z - 1;
        let parent = self.spatial_parent_at_zoom(parent_z).ok()?;
        let next_z = self.z;
        let f_start = parent.f() * 2;
        let x_start = parent.x() * 2;
        let y_start = parent.y() * 2;

        let candidates = [
            SingleId {
                z: next_z,
                f: f_start,
                x: x_start,
                y: y_start,
                temporal_id: parent.temporal().clone(),
            },
            SingleId {
                z: next_z,
                f: f_start,
                x: x_start,
                y: y_start + 1,
                temporal_id: parent.temporal().clone(),
            },
            SingleId {
                z: next_z,
                f: f_start,
                x: x_start + 1,
                y: y_start,
                temporal_id: parent.temporal().clone(),
            },
            SingleId {
                z: next_z,
                f: f_start,
                x: x_start + 1,
                y: y_start + 1,
                temporal_id: parent.temporal().clone(),
            },
            SingleId {
                z: next_z,
                f: f_start + 1,
                x: x_start,
                y: y_start,
                temporal_id: parent.temporal().clone(),
            },
            SingleId {
                z: next_z,
                f: f_start + 1,
                x: x_start,
                y: y_start + 1,
                temporal_id: parent.temporal().clone(),
            },
            SingleId {
                z: next_z,
                f: f_start + 1,
                x: x_start + 1,
                y: y_start,
                temporal_id: parent.temporal().clone(),
            },
            SingleId {
                z: next_z,
                f: f_start + 1,
                x: x_start + 1,
                y: y_start + 1,
                temporal_id: parent.temporal().clone(),
            },
        ];

        let mut siblings: [Option<SingleId>; 7] = [None, None, None, None, None, None, None];
        let mut index = 0;

        for child in candidates {
            if child != *self {
                siblings[index] = Some(child);
                index += 1;
            }
        }

        debug_assert_eq!(index, 7, "sibling set should contain exactly 7 items");
        if index != 7 {
            return None;
        }

        let [a, b, c, d, e, f, g] = siblings;
        match (a, b, c, d, e, f, g) {
            (Some(a), Some(b), Some(c), Some(d), Some(e), Some(f), Some(g)) => {
                Some([a, b, c, d, e, f, g])
            }
            _ => None,
        }
    }

    /// この [`SingleId`] の全ての親を、直近の親から順に列挙する。
    ///
    /// 返す順序は `z - 1`, `z - 2`, ..., `0` であり、元の [`SingleId`] 自身は含まない。
    /// `z = 0` の場合は空のイテレータを返す。
    ///
    /// # 動作例
    ///
    /// 親の列挙:
    /// ```
    /// # use kasane_logic::SingleId;
    /// let id = SingleId::new(3, 3, 2, 7).unwrap();
    /// let parents: Vec<_> = id.spatial_parents().collect();
    ///
    /// assert_eq!(parents.len(), 3);
    /// assert_eq!(parents[0], SingleId::new(2, 1, 1, 3).unwrap());
    /// assert_eq!(parents[1], SingleId::new(1, 0, 0, 1).unwrap());
    /// assert_eq!(parents[2], SingleId::new(0, 0, 0, 0).unwrap());
    /// ```
    pub fn spatial_parents(&self) -> impl Iterator<Item = SingleId> + '_ {
        (0..self.z).rev().map(move |target_z| {
            self.spatial_parent_at_zoom(target_z)
                .expect("target_z must be valid")
        })
    }

    /// この [`SingleId`] の 6 近傍を列挙します。
    ///
    /// 近傍は `f` / `x` / `y` の各軸について `±1` だけ動かした 6 個です。
    /// `x` は循環するため常に有効ですが、`f` と `y` が範囲外になる方向は除外されます。
    /// そのため、境界上の [`SingleId`] では 6 個未満になることがあります。
    ///
    /// ```
    /// # use kasane_logic::SingleId;
    /// let id = SingleId::new(4, 6, 9, 10).unwrap();
    /// let neighbors: Vec<_> = id.spatial_neighbors_6().collect();
    /// assert_eq!(neighbors.len(), 6);
    /// ```
    pub fn spatial_neighbors_6(&self) -> impl Iterator<Item = SingleId> + '_ {
        const OFFSETS: [(i32, i32, i32); 6] = [
            (1, 0, 0),
            (-1, 0, 0),
            (0, 1, 0),
            (0, -1, 0),
            (0, 0, 1),
            (0, 0, -1),
        ];

        OFFSETS.into_iter().filter_map(move |(df, dx, dy)| {
            let mut neighbor = self.clone();

            if df != 0 && neighbor.move_f(df).is_err() {
                return None;
            }

            if dx != 0 {
                neighbor.move_x(dx);
            }

            if dy != 0 && neighbor.move_y(dy).is_err() {
                return None;
            }

            Some(neighbor)
        })
    }

    /// この [`SingleId`] の 26 近傍を列挙します。
    ///
    /// 近傍は `f` / `x` / `y` の各軸について `-1, 0, 1` の組み合わせから
    /// 自身を除いた 26 個です。`x` は循環するため常に有効ですが、`f` と `y` が
    /// 範囲外になる方向は除外されます。
    /// そのため、境界上の [`SingleId`] では 26 個未満になることがあります。
    ///
    /// ```
    /// # use kasane_logic::SingleId;
    /// let id = SingleId::new(4, 6, 9, 10).unwrap();
    /// let neighbors: Vec<_> = id.spatial_neighbors_26().collect();
    /// assert_eq!(neighbors.len(), 26);
    /// ```
    pub fn spatial_neighbors_26(&self) -> impl Iterator<Item = SingleId> + '_ {
        (-1..=1).flat_map(move |df| {
            (-1..=1).flat_map(move |dy| {
                (-1..=1).filter_map(move |dx| {
                    if df == 0 && dx == 0 && dy == 0 {
                        return None;
                    }

                    let mut neighbor = self.clone();

                    if df != 0 && neighbor.move_f(df).is_err() {
                        return None;
                    }

                    if dx != 0 {
                        neighbor.move_x(dx);
                    }

                    if dy != 0 && neighbor.move_y(dy).is_err() {
                        return None;
                    }

                    Some(neighbor)
                })
            })
        })
    }
}
