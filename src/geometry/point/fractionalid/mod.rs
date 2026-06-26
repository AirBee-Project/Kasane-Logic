use crate::spatial_id::zoom_level::{IntoZoomLevel, ZoomLevel};
pub mod impls;

use crate::{
    SingleId,
    error::{Error, GeometryError},
};

/// 実数値のインデックス（Z, F, X, Y）で表される空間 ID の座標型。
///
/// 空間 ID (`SingleId`) が整数値のインデックスを持つのに対し、`FractionalId` は実数値（`f64`）の
/// F、X、Y インデックスを保持することで、グリッド内のより詳細な位置や端点などを表現できます。
///
/// ```
/// # use kasane_logic::ZoomLevel;
/// pub struct FractionalId {
///     z: ZoomLevel,
///     f: f64,
///     x: f64,
///     y: f64,
/// }
/// ```
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct FractionalId {
    z: ZoomLevel,
    f: f64,
    x: f64,
    y: f64,
}

impl FractionalId {
    /// 指定されたズームレベル、F、X、Y の各成分（実数値）から [`FractionalId`] を生成する。
    ///
    /// 各引数は、空間 ID 上で扱える座標インデックスとして有効な範囲に収まっている必要があります。
    /// 範囲外の値が指定された場合、この関数は対応するエラーを返します。
    ///
    /// # 引数
    /// * `z` - 空間 ID のズームレベル（0 〜 [`crate::ZoomLevel::MAX`]）
    /// * `f` - F インデックス（`z.f_min()` 〜 `z.f_max() + 1`）
    /// * `x` - X インデックス（`0.0` 〜 `z.xy_max() + 1`）
    /// * `y` - Y インデックス（`0.0` 〜 `z.xy_max() + 1`）
    ///
    /// # 戻り値
    /// * 有効な値が指定された場合は `Ok(FractionalId)` を返す。
    /// * いずれかの値が範囲外の場合は、対応する `Error` を返す。
    ///
    /// # Examples
    /// ```
    /// # use kasane_logic::FractionalId;
    ///
    /// let fid = FractionalId::new(4, 5.5, 6.2, 7.8).unwrap();
    /// assert_eq!(fid.z(), 4);
    /// assert_eq!(fid.f(), 5.5);
    /// assert_eq!(fid.x(), 6.2);
    /// assert_eq!(fid.y(), 7.8);
    /// ```
    pub fn new(z: impl IntoZoomLevel, f: f64, x: f64, y: f64) -> Result<Self, Error> {
        let zoom = z.into_zoom_level()?;

        // FractionalId は連続値のため、インデックス値の上限ではなく境界までを有効とする。
        // 上端（高度=2^25 / 経度=180° / 緯度の南端）は最後のインデックス値の上面 = 2^z に対応する。
        let f_min = zoom.f_min() as f64;
        let f_max = zoom.f_max() as f64 + 1.0;
        let xy_max = zoom.xy_max() as f64 + 1.0;

        if f < f_min || f > f_max || !f.is_finite() {
            return Err(GeometryError::FractionalFOutOfRange { f, z: zoom.get() }.into());
        }
        if x < 0.0 || x > xy_max || !x.is_finite() {
            return Err(GeometryError::FractionalXOutOfRange { x, z: zoom.get() }.into());
        }
        if y < 0.0 || y > xy_max || !y.is_finite() {
            return Err(GeometryError::FractionalYOutOfRange { y, z: zoom.get() }.into());
        }

        Ok(FractionalId { z: zoom, f, x, y })
    }

    /// ズームレベルを返す。
    ///
    /// # Examples
    /// ```
    /// # use kasane_logic::FractionalId;
    ///
    /// let fid = FractionalId::new(4, 5.5, 6.2, 7.8).unwrap();
    /// assert_eq!(fid.z(), 4);
    /// ```
    pub fn z(&self) -> u8 {
        self.z.get()
    }

    /// F インデックス（実数）を返す。
    ///
    /// # Examples
    /// ```
    /// # use kasane_logic::FractionalId;
    ///
    /// let fid = FractionalId::new(4, 5.5, 6.2, 7.8).unwrap();
    /// assert_eq!(fid.f(), 5.5);
    /// ```
    pub fn f(&self) -> f64 {
        self.f
    }

    /// X インデックス（実数）を返す。
    ///
    /// # Examples
    /// ```
    /// # use kasane_logic::FractionalId;
    ///
    /// let fid = FractionalId::new(4, 5.5, 6.2, 7.8).unwrap();
    /// assert_eq!(fid.x(), 6.2);
    /// ```
    pub fn x(&self) -> f64 {
        self.x
    }

    /// Y インデックス（実数）を返す。
    ///
    /// # Examples
    /// ```
    /// # use kasane_logic::FractionalId;
    ///
    /// let fid = FractionalId::new(4, 5.5, 6.2, 7.8).unwrap();
    /// assert_eq!(fid.y(), 7.8);
    /// ```
    pub fn y(&self) -> f64 {
        self.y
    }

    /// F インデックスを更新する。
    ///
    /// 与えられた `value` が、現在のズームレベル `z` に対応する
    /// `z.f_min()..=z.f_max()` の範囲内にあるかを検証し、範囲外の場合は [`Error`] を返す。
    ///
    /// # Examples
    /// ```
    /// # use kasane_logic::FractionalId;
    ///
    /// let mut fid = FractionalId::new(4, 5.5, 6.2, 7.8).unwrap();
    /// fid.set_f(6.0).unwrap();
    /// assert_eq!(fid.f(), 6.0);
    /// ```
    pub fn set_f(&mut self, value: f64) -> Result<(), Error> {
        let z = self.z.get();
        let min = self.z.f_min() as f64;
        let max = self.z.f_max() as f64;
        if value < min || value > max || !value.is_finite() {
            return Err(GeometryError::FractionalFOutOfRange { f: value, z }.into());
        }
        self.f = value;
        Ok(())
    }

    /// X インデックスを更新する。
    ///
    /// 与えられた `value` が、現在のズームレベル `z` に対応する
    /// `0.0..=z.xy_max()` の範囲内にあるかを検証し、範囲外の場合は [`Error`] を返す。
    ///
    /// # Examples
    /// ```
    /// # use kasane_logic::FractionalId;
    ///
    /// let mut fid = FractionalId::new(4, 5.5, 6.2, 7.8).unwrap();
    /// fid.set_x(8.0).unwrap();
    /// assert_eq!(fid.x(), 8.0);
    /// ```
    pub fn set_x(&mut self, value: f64) -> Result<(), Error> {
        let z = self.z.get();
        let max = self.z.xy_max() as f64;
        if value < 0.0 || value > max || !value.is_finite() {
            return Err(GeometryError::FractionalXOutOfRange { x: value, z }.into());
        }
        self.x = value;
        Ok(())
    }

    /// Y インデックスを更新する。
    ///
    /// 与えられた `value` が、現在のズームレベル `z` に対応する
    /// `0.0..=z.xy_max()` の範囲内にあるかを検証し、範囲外の場合は [`Error`] を返す。
    ///
    /// # Examples
    /// ```
    /// # use kasane_logic::FractionalId;
    ///
    /// let mut fid = FractionalId::new(4, 5.5, 6.2, 7.8).unwrap();
    /// fid.set_y(2.0).unwrap();
    /// assert_eq!(fid.y(), 2.0);
    /// ```
    pub fn set_y(&mut self, value: f64) -> Result<(), Error> {
        let z = self.z.get();
        let max = self.z.xy_max() as f64;
        if value < 0.0 || value > max || !value.is_finite() {
            return Err(GeometryError::FractionalYOutOfRange { y: value, z }.into());
        }
        self.y = value;
        Ok(())
    }

    /// この `FractionalId` が属する、整数値インデックスの [`SingleId`] を返す。
    ///
    /// 内部的には各実数値インデックス（F, X, Y）の床関数（`floor`）を取ることで
    /// 対応する [`SingleId`] を計算します。
    ///
    /// # Examples
    /// ```
    /// # use kasane_logic::{FractionalId, SingleId};
    ///
    /// let fid = FractionalId::new(4, 5.5, 6.2, 7.8).unwrap();
    /// let sid = fid.single_id();
    /// assert_eq!(sid.z(), 4);
    /// assert_eq!(sid.f(), 5);
    /// assert_eq!(sid.x(), 6);
    /// assert_eq!(sid.y(), 7);
    /// ```
    pub fn single_id(&self) -> SingleId {
        SingleId::new(
            self.z.get(),
            libm::floor(self.f) as i32,
            libm::floor(self.x) as u32,
            libm::floor(self.y) as u32,
        )
        .unwrap()
    }

    /// 値の妥当性検証を行わずに [`FractionalId`] を生成する。
    ///
    /// この関数はズームレベルや各実数インデックスに対する範囲チェックを一切行いません。
    /// 呼び出し側は、渡す値が空間 ID の有効な範囲に収まっていることを保証する責任を負います。
    ///
    /// # Safety
    /// この関数は `unsafe` です。
    ///
    /// 以下の制約が保証されない場合、パニック、不正メモリアクセス、未定義動作、
    /// または論理的な不整合を引き起こす可能性があります：
    /// * `z` が有効なズームレベル（0 〜 [`crate::ZoomLevel::MAX`]）であること
    /// * `f` が `z.f_min()..=z.f_max()` の範囲内であること
    /// * `x` および `y` が `0.0..=z.xy_max()` の範囲内であること
    pub unsafe fn new_unchecked(z: u8, f: f64, x: f64, y: f64) -> Self {
        FractionalId {
            z: ZoomLevel::new(z).unwrap(),
            f,
            x,
            y,
        }
    }
}

#[cfg(test)]
mod tests;
