use alloc::string::ToString;
use alloc::vec::Vec;

use crate::{
    FlexId, RangeId, SpatialIdError, TemporalId,
    error::Error,
    spatial_id::constants::{F_MAX, F_MIN, MAX_ZOOM_LEVEL, XY_MAX},
};

impl RangeId {
    /// 与えられた [`FlexId`] 群すべてを包む最小の [`RangeId`]（F/X/Y の3次元AABB）を返す。
    ///
    /// 各セルを全体の最深ズーム（全セルのズームの最大値）へ正規化したうえで、F/X/Y それぞれの
    /// 最小・最大をとる。返り値の各次元の `[0]` が最小（左下）側、`[1]` が最大（右上）側の角に
    /// 対応する。入力が空のときは [`None`] を返す。
    pub(crate) fn bounding_box_of(ids: impl Iterator<Item = FlexId>) -> Option<RangeId> {
        let ranges: Vec<RangeId> = ids.map(|id| RangeId::from(&id)).collect();
        let max_z = ranges.iter().map(RangeId::z).max()?;

        let mut f = [i32::MAX, i32::MIN];
        let mut x = [u32::MAX, u32::MIN];
        let mut y = [u32::MAX, u32::MIN];

        for range in ranges {
            let range = if range.z() == max_z {
                range
            } else {
                range
                    .spatial_children_at_zoom(max_z)
                    .expect("max_z は全セルのズーム以上なので細分化は必ず成功する")
            };
            let (rf, rx, ry) = (range.f(), range.x(), range.y());
            f[0] = f[0].min(rf[0]);
            f[1] = f[1].max(rf[1]);
            x[0] = x[0].min(rx[0]);
            x[1] = x[1].max(rx[1]);
            y[0] = y[0].min(ry[0]);
            y[1] = y[1].max(ry[1]);
        }

        Some(RangeId::new(max_z, f, x, y).expect("正規化済みセルから導いた範囲は max_z で有効"))
    }

    /// 指定された値から [`RangeId`] を構築します。
    /// 与えられた `z`, `f1`, `f2`, `x1`, `x2`, `y1`, `y2` が  各ズームレベルにおける範囲内にあるかを検証し、範囲外の場合は [`Error`] を返します。
    ///
    ///　**各次元の与えられた2つの値は自動的に昇順に並び替えられ、**
    /// **常に `[min, max]` の形で内部に保持されます。**
    ///
    ///
    /// # パラメータ
    /// * `z` — ズームレベル（0–63の範囲が有効）  
    /// * `f1` — 鉛直方向範囲の端のFインデックス
    /// * `f2` — 鉛直方向範囲の端のFインデックス
    /// * `x1` — 東西方向範囲の端のXインデックス
    /// * `x2` — 東西方向範囲の端のXインデックス
    /// * `y1` — 南北方向範囲の端のYインデックス
    /// * `y2` — 南北方向範囲の端のYインデックス
    ///
    /// # バリデーション
    /// - `z` が 63 を超える場合、[`SpatialIdError::ZOutOfRange`] を返します。  
    /// - `f1`,`f2` がズームレベル `z` に対する `F_MIN[z]..=F_MAX[z]` の範囲外の場合、  
    ///   [`SpatialIdError::FOutOfRange`] を返します。  
    /// - `x1`,`x2` または `y1`,`y2` が `0..=XY_MAX[z]` の範囲外の場合、  
    ///   それぞれ [`SpatialIdError::XOutOfRange`]、[`SpatialIdError::YOutOfRange`] を返します。
    ///
    ///
    /// IDの作成:
    /// ```
    /// # use kasane_logic::RangeId;
    /// let id = RangeId::new(4, [-3,6], [8,9], [5,10]).unwrap();
    /// let s = format!("{}", id);
    /// assert_eq!(s, "4/-3:6/8:9/5:10");
    /// ```
    ///
    /// 次元の範囲外の検知:
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::SpatialIdError;
    /// let id = RangeId::new(4, [-3,29], [8,9], [5,10]);
    /// assert_eq!(id, Err(SpatialIdError::FOutOfRange{z:4,f:29}.into()));
    /// ```
    ///
    /// ズームレベルの範囲外の検知:
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::SpatialIdError;
    /// let id = RangeId::new(68, [-3,29], [8,9], [5,10]);
    /// assert_eq!(id, Err(SpatialIdError::ZOutOfRange { z:68 }.into()));
    /// ```
    pub fn new(z: u8, f: [i32; 2], x: [u32; 2], y: [u32; 2]) -> Result<RangeId, Error> {
        if z as usize > MAX_ZOOM_LEVEL {
            return Err(SpatialIdError::ZOutOfRange { z }.into());
        }

        let f_min = F_MIN[z as usize];
        let f_max = F_MAX[z as usize];
        let xy_max = XY_MAX[z as usize];
        let mut f = f;
        let mut y = y;

        for i in 0..2 {
            if f[i] < f_min || f[i] > f_max {
                return Err(SpatialIdError::FOutOfRange { f: f[i], z }.into());
            }
            if x[i] > xy_max {
                return Err(SpatialIdError::XOutOfRange { x: x[i], z }.into());
            }
            if y[i] > xy_max {
                return Err(SpatialIdError::YOutOfRange { y: y[i], z }.into());
            }
        }

        if f[0] > f[1] {
            f.swap(0, 1);
        }
        if y[0] > y[1] {
            y.swap(0, 1);
        }

        Ok(RangeId {
            z,
            f,
            x,
            y,
            temporal_id: TemporalId::WHOLE,
        })
    }

    /// 検証を行わずに [`RangeId`] を構築します。
    ///
    /// この関数は [`RangeId::new`] と異なり、与えられた `z`, `f1`, `f2`, `x1`,`x2`, `y1, `y2` に対して
    /// 一切の範囲チェックや整合性チェックを行いません。
    /// そのため、高速に ID を生成できますが、**不正なパラメータを与えた場合の動作は未定義です**。
    ///
    /// # 注意
    /// 呼び出し側は、以下をすべて満たすことを保証しなければなりません。
    ///
    /// * `z` が有効なズームレベル（0–63）であること  
    /// * `f1`,`f2` が与えられた `z` に応じて `F_MIN[z]..=F_MAX[z]` の範囲内であること  
    /// * `x1`,`x2` および `y1`,`y2` が `0..=XY_MAX[z]` の範囲内であること  
    ///
    /// これらが保証されない場合、本構造体の他のメソッド（範囲を前提とした計算）が
    /// パニック・不正メモリアクセス・未定義動作を引き起こす可能性があります。
    ///
    /// ```
    /// # use kasane_logic::RangeId;
    /// // パラメータが妥当であることを呼び出し側が保証する必要がある
    /// let id = unsafe { RangeId::new_unchecked(5, [-10,-5], [8,9], [5,10]) };
    ///
    /// assert_eq!(id.z(), 5);
    /// assert_eq!(id.f(), [-10,-5]);
    /// assert_eq!(id.x(), [8,9]);
    /// assert_eq!(id.y(), [5,10]);
    /// ```
    /// # Safety
    /// 呼び出し側は、`z` と各次元の配列が対応する有効範囲内であることを保証しなければなりません。
    pub unsafe fn new_unchecked(z: u8, f: [i32; 2], x: [u32; 2], y: [u32; 2]) -> RangeId {
        RangeId {
            z,
            f,
            x,
            y,
            temporal_id: TemporalId::WHOLE,
        }
    }

    #[cfg(feature = "temporal_id")]
    pub fn new_with_temporal(
        z: u8,
        f: [i32; 2],
        x: [u32; 2],
        y: [u32; 2],
        temporal_id: TemporalId,
    ) -> Result<RangeId, Error> {
        if z as usize > MAX_ZOOM_LEVEL {
            return Err(SpatialIdError::ZOutOfRange { z }.into());
        }

        let f_min = F_MIN[z as usize];
        let f_max = F_MAX[z as usize];
        let xy_max = XY_MAX[z as usize];
        let mut f = f;
        let mut y = y;

        for &f_value in &f {
            if f_value < f_min || f_value > f_max {
                return Err(SpatialIdError::FOutOfRange { f: f_value, z }.into());
            }
        }
        for &x_value in &x {
            if x_value > xy_max {
                return Err(SpatialIdError::XOutOfRange { x: x_value, z }.into());
            }
        }
        for &y_value in &y {
            if y_value > xy_max {
                return Err(SpatialIdError::YOutOfRange { y: y_value, z }.into());
            }
        }

        if f[0] > f[1] {
            f.swap(0, 1);
        }
        if y[0] > y[1] {
            y.swap(0, 1);
        }

        Ok(RangeId {
            z,
            f,
            x,
            y,
            temporal_id,
        })
    }

    /// # Safety
    /// 呼び出し側は、`z` と各次元の配列が対応する有効範囲内であることに加え、`temporal_id` が有効な値であることを保証しなければなりません。
    #[cfg(feature = "temporal_id")]
    pub unsafe fn new_with_temporal_unchecked(
        z: u8,
        f: [i32; 2],
        x: [u32; 2],
        y: [u32; 2],
        temporal_id: TemporalId,
    ) -> RangeId {
        RangeId {
            z,
            f,
            x,
            y,
            temporal_id,
        }
    }
}
