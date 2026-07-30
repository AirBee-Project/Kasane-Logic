use crate::{
    RangeId, TemporalId,
    error::Error,
    spatial_id::{helpers::IntoRange, zoom_level::ZoomLevel},
};

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
    /// * `f` — 鉛直方向範囲。`[f1,f2]`のような`[i32;2]`、または両端が等しい単一の`i32`値
    /// * `x` — 東西方向範囲。`[x1,x2]`のような`[u32;2]`、または両端が等しい単一の`u32`値
    /// * `y` — 南北方向範囲。`[y1,y2]`のような`[u32;2]`、または両端が等しい単一の`u32`値
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
    /// 各次元は単一値でも指定できる（空間的に1点の`RangeId`を簡潔に作れる）:
    /// ```
    /// # use kasane_logic::RangeId;
    /// let id = RangeId::new(4, -3, 8, 5).unwrap();
    /// assert_eq!(id, RangeId::new(4, [-3,-3], [8,8], [5,5]).unwrap());
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
    pub fn new(
        z: impl Into<u8>,
        f: impl IntoRange<i32>,
        x: impl IntoRange<u32>,
        y: impl IntoRange<u32>,
    ) -> Result<RangeId, Error> {
        let zoom = ZoomLevel::new(z.into())?;
        let mut f = f.into_range();
        let x = x.into_range();
        let mut y = y.into_range();

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
            temporal: TemporalId::WHOLE,
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
            temporal: TemporalId::WHOLE,
        }
    }
}
