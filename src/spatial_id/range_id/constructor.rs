use crate::{RangeId, TemporalId, error::Error, spatial_id::zoom_level::ZoomLevel};

impl RangeId {
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
    /// - `z` が 63 を超える場合、[`crate::SpatialIdError::ZOutOfRange`] を返します。  
    /// - `f` が与えられた `z` に応じて有効範囲外である場合、
    ///   [`crate::SpatialIdError::FOutOfRange`] を返します。  
    /// - `x` や `y` が与えられた `z` に応じて有効範囲外である場合、
    ///   それぞれ [`crate::SpatialIdError::XOutOfRange`]、[`crate::SpatialIdError::YOutOfRange`] を返します。
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
    pub fn new(z: impl Into<u8>, f: [i32; 2], x: [u32; 2], y: [u32; 2]) -> Result<RangeId, Error> {
        let zoom = ZoomLevel::new(z.into())?;
        let mut f = f;
        let mut y = y;

        for i in 0..2 {
            zoom.check_f(f[i])?;
            zoom.check_x(x[i])?;
            zoom.check_y(y[i])?;
        }

        if f[0] > f[1] {
            f.swap(0, 1);
        }
        if y[0] > y[1] {
            y.swap(0, 1);
        }

        Ok(RangeId {
            z: zoom,
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
    /// * `f1`,`f2` が与えられた `z` に応じて `ZoomLevel::new(z as u8)?.f_min()..=unsafe { ZoomLevel::new_unchecked(z as u8) }.f_max()` の範囲内であること  
    /// * `x1`,`x2` および `y1`,`y2` が `0..=unsafe { ZoomLevel::new_unchecked(z as u8) }.xy_max()` の範囲内であること  
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
            z: unsafe { ZoomLevel::new_unchecked(z) },
            f,
            x,
            y,
            temporal_id: TemporalId::WHOLE,
        }
    }

    #[cfg(feature = "temporal_id")]
    pub fn new_with_temporal(
        z: impl Into<u8>,
        f: [i32; 2],
        x: [u32; 2],
        y: [u32; 2],
        temporal_id: TemporalId,
    ) -> Result<RangeId, Error> {
        let zoom = ZoomLevel::new(z.into())?;
        let mut f = f;
        let mut y = y;

        for i in 0..2 {
            zoom.check_f(f[i])?;
            zoom.check_x(x[i])?;
            zoom.check_y(y[i])?;
        }

        if f[0] > f[1] {
            f.swap(0, 1);
        }
        if y[0] > y[1] {
            y.swap(0, 1);
        }

        Ok(RangeId {
            z: zoom,
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
            z: unsafe { ZoomLevel::new_unchecked(z) },
            f,
            x,
            y,
            temporal_id,
        }
    }
}
