use alloc::vec::Vec;

pub mod constructor;
pub mod convert;
pub mod encode;
pub mod impls;
pub mod ops;

use crate::{
    Error, Side, SpatialIdError, TemporalId,
    spatial_id::{
        range_id::convert::{split_f, split_xy},
        zoom_level::ZoomLevel,
    },
};

#[derive(Clone, PartialEq, Debug, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
///拡張空間ID
pub struct FlexId {
    f_zoomlevel: ZoomLevel,
    f_index: i32,
    x_zoomlevel: ZoomLevel,
    x_index: u32,
    y_zoomlevel: ZoomLevel,
    y_index: u32,
    temporal_id: TemporalId,
}

impl FlexId {
    pub const UPPER_MAX: FlexId = FlexId {
        f_zoomlevel: ZoomLevel::MIN,
        f_index: 0,
        x_zoomlevel: ZoomLevel::MIN,
        x_index: 0,
        y_zoomlevel: ZoomLevel::MIN,
        y_index: 0,
        temporal_id: TemporalId::WHOLE,
    };

    pub const LOWER_MAX: FlexId = FlexId {
        f_zoomlevel: ZoomLevel::MIN,
        f_index: -1,
        x_zoomlevel: ZoomLevel::MIN,
        x_index: 0,
        y_zoomlevel: ZoomLevel::MIN,
        y_index: 0,
        temporal_id: TemporalId::WHOLE,
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
    pub fn f_index(&self) -> i32 {
        self.f_index
    }
    pub fn x_index(&self) -> u32 {
        self.x_index
    }
    pub fn y_index(&self) -> u32 {
        self.y_index
    }

    /// 空間成分だけを残し、時間を全時間（WHOLE）に置き換えた [`FlexId`] を返す。
    pub fn spatial_part(&self) -> FlexId {
        self.with_temporal(TemporalId::WHOLE)
    }

    /// 空間成分はそのままに、時間IDを `temporal` に置き換えた [`FlexId`] を返す。
    pub fn with_temporal(&self, temporal: TemporalId) -> FlexId {
        FlexId {
            f_zoomlevel: self.f_zoomlevel,
            f_index: self.f_index,
            x_zoomlevel: self.x_zoomlevel,
            x_index: self.x_index,
            y_zoomlevel: self.y_zoomlevel,
            y_index: self.y_index,
            temporal_id: temporal,
        }
    }

    /// このFlexIdを高さ（F）方向へ、ズームレベル `z` のセル `index` 個分だけ平行移動した結果を返す。
    ///
    /// 移動量はズーム `z` を単位とするため、`z` がこのFlexIdのFズームレベルより
    /// 細かい場合は1セルに満たない移動となり、結果が複数セルへ分割されることがある。
    /// そのため複数の [`FlexId`] を生成するイテレーターを返す。XY方向の値は変更しない。
    ///
    /// # バリデーション
    /// - `z` が [`ZoomLevel::MAX`] を超える場合は [`SpatialIdError::ZOutOfRange`] を返す。
    /// - `index` がズーム `z` のF範囲（`ZoomLevel::new(z as u8).unwrap().f_min()..=ZoomLevel::new(z as u8).unwrap().f_max()`）外の場合は
    ///   [`SpatialIdError::FOutOfRange`] を返す。
    /// - 移動後の位置が、両者を合わせたズーム `max(f_zoomlevel, z)` のF範囲を超える場合は
    ///   [`SpatialIdError::FOutOfRange`] を返す。
    pub fn shift_f(&self, z: u8, index: i32) -> Result<impl Iterator<Item = FlexId>, Error> {
        // ズームレベルのチェック
        let zoom = ZoomLevel::new(z)?;
        zoom.check_f(index)?;

        let f_zoomlevel = self.f_zoomlevel();
        let max_z = f_zoomlevel.max(z);

        let cell_scale = 1_i32 << (max_z - f_zoomlevel);
        let delta_index = index * (1_i32 << (max_z - z));

        let left = self.f_index() * cell_scale + delta_index;
        let right = left + cell_scale - 1;

        // 移動後が max_z のF範囲を超える場合はエラー。
        if left < ZoomLevel::new(max_z)?.f_min() {
            return Err(SpatialIdError::FOutOfRange { z: max_z, f: left }.into());
        }
        if right > ZoomLevel::new(max_z)?.f_max() {
            return Err(SpatialIdError::FOutOfRange { z: max_z, f: right }.into());
        }

        // F以外の成分は値で捕捉し、返すイテレーターが self を借用しないようにする。
        let x_zoomlevel = self.x_zoomlevel();
        let x_index = self.x_index();
        let y_zoomlevel = self.y_zoomlevel();
        let y_index = self.y_index();
        let temporal_id = self.temporal_id;

        // 占有区間を整列したセル群へ分解し、F以外の成分を保ったままFlexIdを構築する。
        Ok(
            split_f(max_z, [left, right]).map(move |(seg_z, seg_index)| {
                FlexId::new(seg_z, seg_index, x_zoomlevel, x_index, y_zoomlevel, y_index)
                    .map(|id| id.with_temporal(temporal_id))
                    .unwrap()
            }),
        )
    }

    /// このFlexIdを東西（X）方向へ、ズームレベル `z` のセル `index` 個分だけ平行移動した結果を返す。
    ///
    /// X方向はWebメルカトル図法において東西に巡回するため、移動量がどれだけ大きくても
    /// エラーにはならず、`max(x_zoomlevel, z)` の周長を法として循環する。境界（経度±180度）を
    /// またぐ場合は、[`RangeId`](crate::RangeId) の巡回表現と同様に分割される。
    /// F・Y方向の値は変更しない。
    ///
    /// # バリデーション
    /// - `z` が [`ZoomLevel::MAX`] を超える場合は [`SpatialIdError::ZOutOfRange`] を返す。
    pub fn shift_x(&self, z: u8, index: i32) -> Result<impl Iterator<Item = FlexId>, Error> {
        if z > ZoomLevel::MAX.get() {
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
        let temporal_id = self.temporal_id;

        Ok(x_cells.into_iter().map(move |(seg_z, seg_index)| {
            FlexId::new(f_zoomlevel, f_index, seg_z, seg_index, y_zoomlevel, y_index)
                .map(|id| id.with_temporal(temporal_id))
                .unwrap()
        }))
    }

    /// このFlexIdを南北（Y）方向へ、ズームレベル `z` のセル `index` 個分だけ平行移動した結果を返す。
    ///
    /// Y方向は巡回せず `[0[z]]` に制限される。`z` が このFlexIdのYズームレベルより細かい場合は結果が分割されることがある。F・X方向の値は変更しない。
    ///
    /// # バリデーション
    /// - `z` が [`ZoomLevel::MAX`] を超える場合は [`SpatialIdError::ZOutOfRange`] を返す。
    /// - 移動後の位置が、両者を合わせたズーム `max(y_zoomlevel, z)` のY範囲を超える場合は
    ///   [`SpatialIdError::YOutOfRange`] を返す。
    pub fn shift_y(&self, z: u8, index: i32) -> Result<impl Iterator<Item = FlexId>, Error> {
        if z > ZoomLevel::MAX.get() {
            return Err(SpatialIdError::ZOutOfRange { z }.into());
        }

        let y_zoomlevel = self.y_zoomlevel();
        let max_z = y_zoomlevel.max(z);

        let cell_scale = 1_i64 << (max_z - y_zoomlevel);
        let delta_index = index as i64 * (1_i64 << (max_z - z));

        let left = self.y_index() as i64 * cell_scale + delta_index;
        let right = left + cell_scale - 1;

        // 移動後が max_z のY範囲 [0[max_z]] を超える場合はエラー。
        let y_max = ZoomLevel::new(max_z)?.xy_max() as i64;
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
        let temporal_id = self.temporal_id;

        Ok(
            split_xy(max_z, [left as u32, right as u32]).map(move |(seg_z, seg_index)| {
                FlexId::new(f_zoomlevel, f_index, x_zoomlevel, x_index, seg_z, seg_index)
                    .map(|id| id.with_temporal(temporal_id))
                    .unwrap()
            }),
        )
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
        let temporal_id = self.temporal_id;

        Ok(
            split_f(max_z, [left, right]).map(move |(seg_z, seg_index)| {
                FlexId::new(seg_z, seg_index, x_zoomlevel, x_index, y_zoomlevel, y_index)
                    .map(|id| id.with_temporal(temporal_id))
                    .unwrap()
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

        let x_cells: Vec<(u8, u32)> = ranges
            .into_iter()
            .flat_map(|range| split_xy(max_z, range))
            .collect();

        let f_zoomlevel = self.f_zoomlevel();
        let f_index = self.f_index();
        let y_zoomlevel = self.y_zoomlevel();
        let y_index = self.y_index();
        let temporal_id = self.temporal_id;

        Ok(x_cells.into_iter().map(move |(seg_z, seg_index)| {
            FlexId::new(f_zoomlevel, f_index, seg_z, seg_index, y_zoomlevel, y_index)
                .map(|id| id.with_temporal(temporal_id))
                .unwrap()
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
        let temporal_id = self.temporal_id;

        Ok(
            split_xy(max_z, [left as u32, right as u32]).map(move |(seg_z, seg_index)| {
                FlexId::new(f_zoomlevel, f_index, x_zoomlevel, x_index, seg_z, seg_index)
                    .map(|id| id.with_temporal(temporal_id))
                    .unwrap()
            }),
        )
    }

    /// このFlexIdのF方向の占有を、ズーム `z` の絶対座標範囲 `[lo, hi]` に置き換える。
    ///
    /// [`stretch_f`](Self::stretch_f) が元の占有を相対的に継ぎ足して起伏を保存するのに対し、
    /// `level_f` は元のF位置を捨てて全セルを同じ絶対範囲 `[lo, hi]` に揃える。よってF方向の
    /// 起伏（凹凸）は平坦化され、`hi` を越えていた占有は削られ、足りない区間は埋められる。
    /// `lo`/`hi` は順不同で、内部で小さい方を下端として扱う。XY方向の値は変更しない。
    /// 占有区間は整列したセル群へ分解されるため複数の [`FlexId`] を返す。
    ///
    /// # バリデーション
    /// - `z` が [`ZoomLevel::MAX`] を超える場合は [`SpatialIdError::ZOutOfRange`] を返す。
    /// - `lo` または `hi` がズーム `z` のF範囲（`ZoomLevel::new(z as u8).unwrap().f_min()..=ZoomLevel::new(z as u8).unwrap().f_max()`）外の場合は
    ///   [`SpatialIdError::FOutOfRange`] を返す。
    pub fn level_f(&self, z: u8, lo: i32, hi: i32) -> Result<impl Iterator<Item = FlexId>, Error> {
        if z > ZoomLevel::MAX.get() {
            return Err(SpatialIdError::ZOutOfRange { z }.into());
        }

        let (left, right) = (lo.min(hi), lo.max(hi));
        if left < ZoomLevel::new(z).unwrap().f_min() {
            return Err(SpatialIdError::FOutOfRange { z, f: left }.into());
        }
        if right > ZoomLevel::new(z).unwrap().f_max() {
            return Err(SpatialIdError::FOutOfRange { z, f: right }.into());
        }

        let x_zoomlevel = self.x_zoomlevel();
        let x_index = self.x_index();
        let y_zoomlevel = self.y_zoomlevel();
        let y_index = self.y_index();
        let temporal_id = self.temporal_id;

        Ok(split_f(z, [left, right]).map(move |(seg_z, seg_index)| {
            FlexId::new(seg_z, seg_index, x_zoomlevel, x_index, y_zoomlevel, y_index)
                .map(|id| id.with_temporal(temporal_id))
                .unwrap()
        }))
    }

    /// このFlexIdのX方向の占有を、ズーム `z` の絶対座標範囲（`from` から東向きに `to` まで）へ
    /// 置き換える。
    ///
    /// [`level_f`](Self::level_f) のX版だが、X方向は東西に巡回するため `from` から東向きに
    /// `to` まで進む弧として解釈する。`from <= to` なら連続した一区間、`from > to` なら境界を
    /// またいで2区間に分割される。元のX位置は捨てられ、X方向の起伏は平坦化される。
    /// F・Y方向の値は変更しない。
    ///
    /// # バリデーション
    /// - `z` が [`ZoomLevel::MAX`] を超える場合は [`SpatialIdError::ZOutOfRange`] を返す。
    /// - `from` または `to` がズーム `z` のX範囲（`0..=ZoomLevel::new(z as u8).unwrap().xy_max()`）外の場合は
    ///   [`SpatialIdError::XOutOfRange`] を返す。
    pub fn level_x(
        &self,
        z: u8,
        from: u32,
        to: u32,
    ) -> Result<impl Iterator<Item = FlexId>, Error> {
        if z > ZoomLevel::MAX.get() {
            return Err(SpatialIdError::ZOutOfRange { z }.into());
        }

        let xy_max = ZoomLevel::new(z).unwrap().xy_max();
        if from > xy_max {
            return Err(SpatialIdError::XOutOfRange { z, x: from }.into());
        }
        if to > xy_max {
            return Err(SpatialIdError::XOutOfRange { z, x: to }.into());
        }

        // from から東向きに to まで。境界跨ぎ（from > to）は2区間へ分ける。
        let ranges: Vec<[u32; 2]> = if from <= to {
            vec![[from, to]]
        } else {
            vec![[from, xy_max], [0, to]]
        };
        let x_cells: Vec<(u8, u32)> = ranges
            .into_iter()
            .flat_map(|range| split_xy(z, range))
            .collect();

        let f_zoomlevel = self.f_zoomlevel();
        let f_index = self.f_index();
        let y_zoomlevel = self.y_zoomlevel();
        let y_index = self.y_index();
        let temporal_id = self.temporal_id;

        Ok(x_cells.into_iter().map(move |(seg_z, seg_index)| {
            FlexId::new(f_zoomlevel, f_index, seg_z, seg_index, y_zoomlevel, y_index)
                .map(|id| id.with_temporal(temporal_id))
                .unwrap()
        }))
    }

    /// このFlexIdのY方向の占有を、ズーム `z` の絶対座標範囲 `[lo, hi]` に置き換える。
    ///
    /// [`level_f`](Self::level_f) のY版。Y方向は巡回せず `[0[z]]` に制限される。
    /// `lo`/`hi` は順不同で、内部で小さい方を下端として扱う。元のY位置は捨てられ、Y方向の
    /// 起伏は平坦化される。F・X方向の値は変更しない。
    ///
    /// # バリデーション
    /// - `z` が [`ZoomLevel::MAX`] を超える場合は [`SpatialIdError::ZOutOfRange`] を返す。
    /// - `lo` または `hi` がズーム `z` のY範囲（`0..=ZoomLevel::new(z as u8).unwrap().xy_max()`）外の場合は
    ///   [`SpatialIdError::YOutOfRange`] を返す。
    pub fn level_y(&self, z: u8, lo: u32, hi: u32) -> Result<impl Iterator<Item = FlexId>, Error> {
        if z > ZoomLevel::MAX.get() {
            return Err(SpatialIdError::ZOutOfRange { z }.into());
        }

        let (left, right) = (lo.min(hi), lo.max(hi));
        if right > ZoomLevel::new(z).unwrap().xy_max() {
            return Err(SpatialIdError::YOutOfRange { z, y: right }.into());
        }

        let f_zoomlevel = self.f_zoomlevel();
        let f_index = self.f_index();
        let x_zoomlevel = self.x_zoomlevel();
        let x_index = self.x_index();
        let temporal_id = self.temporal_id;

        Ok(split_xy(z, [left, right]).map(move |(seg_z, seg_index)| {
            FlexId::new(f_zoomlevel, f_index, x_zoomlevel, x_index, seg_z, seg_index)
                .map(|id| id.with_temporal(temporal_id))
                .unwrap()
        }))
    }

    ///F方向で二つに切り分ける
    pub fn split_f(&self, side: Side) -> Option<FlexId> {
        if self.f_zoomlevel() == ZoomLevel::MAX.get() {
            None
        } else {
            Some(
                FlexId::new(
                    self.f_zoomlevel() + 1,
                    self.f_index() * 2 + side as i32,
                    self.x_zoomlevel(),
                    self.x_index(),
                    self.y_zoomlevel(),
                    self.y_index(),
                )
                .map(|id| id.with_temporal(self.temporal_id))
                .unwrap(),
            )
        }
    }

    ///X方向で二つに切り分ける
    pub fn split_x(&self, side: Side) -> Option<FlexId> {
        if self.x_zoomlevel() == ZoomLevel::MAX.get() {
            None
        } else {
            Some(
                FlexId::new(
                    self.f_zoomlevel(),
                    self.f_index(),
                    self.x_zoomlevel() + 1,
                    self.x_index() * 2 + side as u32,
                    self.y_zoomlevel(),
                    self.y_index(),
                )
                .map(|id| id.with_temporal(self.temporal_id))
                .unwrap(),
            )
        }
    }

    ///Y方向で二つに切り分ける
    pub fn split_y(&self, side: Side) -> Option<FlexId> {
        if self.y_zoomlevel() == ZoomLevel::MAX.get() {
            None
        } else {
            Some(
                FlexId::new(
                    self.f_zoomlevel(),
                    self.f_index(),
                    self.x_zoomlevel(),
                    self.x_index(),
                    self.y_zoomlevel() + 1,
                    self.y_index() * 2 + side as u32,
                )
                .map(|id| id.with_temporal(self.temporal_id))
                .unwrap(),
            )
        }
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

        let temporal_id = self.temporal_id;
        (f_start..=f_end).flat_map(move |f_idx| {
            let temp_id1 = temporal_id;
            (x_start..=x_end).flat_map(move |x_idx| {
                let temp_id2 = temp_id1;
                (y_start..=y_end).map(move |y_idx| {
                    let seg_fi = if sz_f >= tz_f { self_fi } else { f_idx };
                    let seg_xi = if sz_x >= tz_x { self_xi } else { x_idx };
                    let seg_yi = if sz_y >= tz_y { self_yi } else { y_idx };

                    let parent = FlexId::new(tz_f, f_idx, tz_x, x_idx, tz_y, y_idx)
                        .map(|id| id.with_temporal(temp_id2))
                        .unwrap();

                    let seg = FlexId::new(seg_fz, seg_fi, seg_xz, seg_xi, seg_yz, seg_yi)
                        .map(|id| id.with_temporal(temp_id2))
                        .unwrap();

                    (parent, seg)
                })
            })
        })
    }
}
