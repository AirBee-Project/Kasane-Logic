pub mod constructor;
pub mod convert;
pub mod impls;
pub mod ops;

use crate::{
    Error, F_MAX, F_MIN, MAX_ZOOM_LEVEL, Side, SpatialIdError, TemporalId, XY_MAX,
    spatial_id::range_id::convert::{split_f, split_xy},
};

#[derive(Clone, PartialEq, Debug, Eq, PartialOrd, Ord, Hash)]
///拡張空間ID
pub struct FlexId {
    f_zoomlevel: u8,
    f_index: i32,
    x_zoomlevel: u8,
    x_index: u32,
    y_zoomlevel: u8,
    y_index: u32,
    temporal_id: TemporalId,
}

impl FlexId {
    pub const UPPER_MAX: FlexId = FlexId {
        f_zoomlevel: 0,
        f_index: 0,
        x_zoomlevel: 0,
        x_index: 0,
        y_zoomlevel: 0,
        y_index: 0,
        temporal_id: TemporalId::WHOLE,
    };

    pub const LOWER_MAX: FlexId = FlexId {
        f_zoomlevel: 0,
        f_index: -1,
        x_zoomlevel: 0,
        x_index: 0,
        y_zoomlevel: 0,
        y_index: 0,
        temporal_id: TemporalId::WHOLE,
    };

    pub fn f_zoomlevel(&self) -> u8 {
        self.f_zoomlevel
    }
    pub fn x_zoomlevel(&self) -> u8 {
        self.x_zoomlevel
    }
    pub fn y_zoomlevel(&self) -> u8 {
        self.y_zoomlevel
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

    /// このFlexIdを高さ（F）方向へ、ズームレベル `z` のセル `index` 個分だけ平行移動した結果を返す。
    ///
    /// 移動量はズーム `z` を単位とするため、`z` がこのFlexIdのFズームレベルより
    /// 細かい場合は1セルに満たない移動となり、結果が複数セルへ分割されることがある。
    /// そのため複数の [`FlexId`] を生成するイテレーターを返す。XY方向の値は変更しない。
    ///
    /// # バリデーション
    /// - `z` が [`MAX_ZOOM_LEVEL`] を超える場合は [`SpatialIdError::ZOutOfRange`] を返す。
    /// - `index` がズーム `z` のF範囲（`F_MIN[z]..=F_MAX[z]`）外の場合は
    ///   [`SpatialIdError::FOutOfRange`] を返す。
    /// - 移動後の位置が、両者を合わせたズーム `max(f_zoomlevel, z)` のF範囲を超える場合は
    ///   [`SpatialIdError::FOutOfRange`] を返す。
    pub fn shift_f(&self, z: u8, index: i32) -> Result<impl Iterator<Item = FlexId>, Error> {
        // ズームレベルのチェック
        if z > MAX_ZOOM_LEVEL as u8 {
            return Err(Error::SpatialId(SpatialIdError::ZOutOfRange { z }));
        };

        // 移動インデックス値のチェック
        if index < F_MIN[z as usize] || index > F_MAX[z as usize] {
            return Err(SpatialIdError::FOutOfRange { z, f: index }.into());
        };

        let f_zoomlevel = self.f_zoomlevel();
        let max_z = f_zoomlevel.max(z);

        let cell_scale = 1_i32 << (max_z - f_zoomlevel);
        let delta_index = index * (1_i32 << (max_z - z));

        let left = self.f_index() * cell_scale + delta_index;
        let right = left + cell_scale - 1;

        // 移動後が max_z のF範囲を超える場合はエラー。
        if left < F_MIN[max_z as usize] {
            return Err(SpatialIdError::FOutOfRange { z: max_z, f: left }.into());
        }
        if right > F_MAX[max_z as usize] {
            return Err(SpatialIdError::FOutOfRange { z: max_z, f: right }.into());
        }

        // F以外の成分は値で捕捉し、返すイテレーターが self を借用しないようにする。
        let x_zoomlevel = self.x_zoomlevel();
        let x_index = self.x_index();
        let y_zoomlevel = self.y_zoomlevel();
        let y_index = self.y_index();
        #[cfg(feature = "temporal_id")]
        let temporal_id = self.temporal_id.clone();

        // 占有区間を整列したセル群へ分解し、F以外の成分を保ったままFlexIdを構築する。
        Ok(
            split_f(max_z, [left, right]).map(move |(seg_z, seg_index)| {
                #[cfg(feature = "temporal_id")]
                {
                    unsafe {
                        FlexId::new_with_temporal_unchecked(
                            seg_z,
                            seg_index,
                            x_zoomlevel,
                            x_index,
                            y_zoomlevel,
                            y_index,
                            temporal_id.clone(),
                        )
                    }
                }

                #[cfg(not(feature = "temporal_id"))]
                {
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
                }
            }),
        )
    }

    /// このFlexIdを東西（X）方向へ、ズームレベル `z` のセル `index` 個分だけ平行移動した 結果を返す。
    ///
    /// X方向はWebメルカトル図法において東西に巡回するため、移動量がどれだけ大きくても
    /// エラーにはならず、`max(x_zoomlevel, z)` の周長を法として循環する。境界（経度±180度）を またぐ場合は、[`RangeId`](crate::RangeId) の巡回表現と同様に分割される。
    /// F・Y方向の値は変更しない。
    ///
    /// # バリデーション
    /// - `z` が [`MAX_ZOOM_LEVEL`] を超える場合は [`SpatialIdError::ZOutOfRange`] を返す。
    pub fn shift_x(&self, z: u8, index: i32) -> Result<impl Iterator<Item = FlexId>, Error> {
        if z > MAX_ZOOM_LEVEL as u8 {
            return Err(SpatialIdError::ZOutOfRange { z }.into());
        }

        let x_zoomlevel = self.x_zoomlevel();
        let max_z = x_zoomlevel.max(z);

        // max_z における周長（Xセル数）。
        let circumference = 1_i64 << max_z;
        let cell_scale = 1_i64 << (max_z - x_zoomlevel);
        let delta_index = index as i64 * (1_i64 << (max_z - z));

        let left = self.x_index() as i64 * cell_scale + delta_index;
        let right = left + cell_scale - 1;

        let left_wrapped = left.rem_euclid(circumference);
        let right_wrapped = right.rem_euclid(circumference);
        let ranges: Vec<[u32; 2]> = if left_wrapped <= right_wrapped {
            vec![[left_wrapped as u32, right_wrapped as u32]]
        } else {
            vec![
                [left_wrapped as u32, (circumference - 1) as u32],
                [0, right_wrapped as u32],
            ]
        };

        let x_cells: Vec<(u8, u32)> = ranges
            .into_iter()
            .flat_map(|range| split_xy(max_z, range))
            .collect();

        let f_zoomlevel = self.f_zoomlevel();
        let f_index = self.f_index();
        let y_zoomlevel = self.y_zoomlevel();
        let y_index = self.y_index();
        #[cfg(feature = "temporal_id")]
        let temporal_id = self.temporal_id.clone();

        Ok(x_cells.into_iter().map(move |(seg_z, seg_index)| {
            #[cfg(feature = "temporal_id")]
            {
                unsafe {
                    FlexId::new_with_temporal_unchecked(
                        f_zoomlevel,
                        f_index,
                        seg_z,
                        seg_index,
                        y_zoomlevel,
                        y_index,
                        temporal_id.clone(),
                    )
                }
            }

            #[cfg(not(feature = "temporal_id"))]
            {
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
            }
        }))
    }

    /// このFlexIdを南北（Y）方向へ、ズームレベル `z` のセル `index` 個分だけ平行移動した結果を返す。
    ///
    /// Y方向は巡回せず `[0, XY_MAX[z]]` に制限される。`z` が このFlexIdのYズームレベルより細かい場合は結果が分割されることがある。F・X方向の値は変更しない。
    ///
    /// # バリデーション
    /// - `z` が [`MAX_ZOOM_LEVEL`] を超える場合は [`SpatialIdError::ZOutOfRange`] を返す。
    /// - 移動後の位置が、両者を合わせたズーム `max(y_zoomlevel, z)` のY範囲を超える場合は
    ///   [`SpatialIdError::YOutOfRange`] を返す。
    pub fn shift_y(&self, z: u8, index: i32) -> Result<impl Iterator<Item = FlexId>, Error> {
        if z > MAX_ZOOM_LEVEL as u8 {
            return Err(SpatialIdError::ZOutOfRange { z }.into());
        }

        let y_zoomlevel = self.y_zoomlevel();
        let max_z = y_zoomlevel.max(z);

        let cell_scale = 1_i64 << (max_z - y_zoomlevel);
        let delta_index = index as i64 * (1_i64 << (max_z - z));

        let left = self.y_index() as i64 * cell_scale + delta_index;
        let right = left + cell_scale - 1;

        // 移動後が max_z のY範囲 [0, XY_MAX[max_z]] を超える場合はエラー。
        let y_max = XY_MAX[max_z as usize] as i64;
        if left < 0 || right > y_max {
            let offending = if left < 0 { left } else { right };
            return Err(SpatialIdError::YOutOfRange {
                z: max_z,
                y: offending.clamp(0, u32::MAX as i64) as u32,
            }
            .into());
        }

        // F・X以外の成分は値で捕捉する。
        let f_zoomlevel = self.f_zoomlevel();
        let f_index = self.f_index();
        let x_zoomlevel = self.x_zoomlevel();
        let x_index = self.x_index();
        #[cfg(feature = "temporal_id")]
        let temporal_id = self.temporal_id.clone();

        Ok(
            split_xy(max_z, [left as u32, right as u32]).map(move |(seg_z, seg_index)| {
                #[cfg(feature = "temporal_id")]
                {
                    unsafe {
                        FlexId::new_with_temporal_unchecked(
                            f_zoomlevel,
                            f_index,
                            x_zoomlevel,
                            x_index,
                            seg_z,
                            seg_index,
                            temporal_id.clone(),
                        )
                    }
                }

                #[cfg(not(feature = "temporal_id"))]
                {
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
                }
            }),
        )
    }

    ///F方向で二つに切り分ける
    pub fn split_f(&self, side: Side) -> Option<FlexId> {
        if self.f_zoomlevel() == MAX_ZOOM_LEVEL as u8 {
            None
        } else {
            #[cfg(feature = "temporal_id")]
            {
                Some(unsafe {
                    FlexId::new_with_temporal_unchecked(
                        self.f_zoomlevel() + 1,
                        self.f_index() * 2 + side as i32,
                        self.x_zoomlevel(),
                        self.x_index(),
                        self.y_zoomlevel(),
                        self.y_index(),
                        self.temporal_id.clone(),
                    )
                })
            }

            #[cfg(not(feature = "temporal_id"))]
            {
                Some(unsafe {
                    FlexId::new_unchecked(
                        self.f_zoomlevel() + 1,
                        self.f_index() * 2 + side as i32,
                        self.x_zoomlevel(),
                        self.x_index(),
                        self.y_zoomlevel(),
                        self.y_index(),
                    )
                })
            }
        }
    }

    ///X方向で二つに切り分ける
    pub fn split_x(&self, side: Side) -> Option<FlexId> {
        if self.x_zoomlevel() == MAX_ZOOM_LEVEL as u8 {
            None
        } else {
            #[cfg(feature = "temporal_id")]
            {
                Some(unsafe {
                    FlexId::new_with_temporal_unchecked(
                        self.f_zoomlevel(),
                        self.f_index(),
                        self.x_zoomlevel() + 1,
                        self.x_index() * 2 + side as u32,
                        self.y_zoomlevel(),
                        self.y_index(),
                        self.temporal_id.clone(),
                    )
                })
            }

            #[cfg(not(feature = "temporal_id"))]
            {
                Some(unsafe {
                    FlexId::new_unchecked(
                        self.f_zoomlevel(),
                        self.f_index(),
                        self.x_zoomlevel() + 1,
                        self.x_index() * 2 + side as u32,
                        self.y_zoomlevel(),
                        self.y_index(),
                    )
                })
            }
        }
    }

    ///Y方向で二つに切り分ける
    pub fn split_y(&self, side: Side) -> Option<FlexId> {
        if self.y_zoomlevel() == MAX_ZOOM_LEVEL as u8 {
            None
        } else {
            #[cfg(feature = "temporal_id")]
            {
                Some(unsafe {
                    FlexId::new_with_temporal_unchecked(
                        self.f_zoomlevel(),
                        self.f_index(),
                        self.x_zoomlevel(),
                        self.x_index(),
                        self.y_zoomlevel() + 1,
                        self.y_index() * 2 + side as u32,
                        self.temporal_id.clone(),
                    )
                })
            }

            #[cfg(not(feature = "temporal_id"))]
            {
                Some(unsafe {
                    FlexId::new_unchecked(
                        self.f_zoomlevel(),
                        self.f_index(),
                        self.x_zoomlevel(),
                        self.x_index(),
                        self.y_zoomlevel() + 1,
                        self.y_index() * 2 + side as u32,
                    )
                })
            }
        }
    }
}
