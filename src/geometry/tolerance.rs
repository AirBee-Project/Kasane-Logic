use core::fmt;

use crate::{Error, GeometryError};

/// 幾何計算における同一点判定などの許容誤差（メートル単位）を表す型。
///
/// 浮動小数点数演算における誤差を吸収し、近接した頂点の同一判定や
/// 閉合性検証に用いる。
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Tolerance(f64);

impl Tolerance {
    /// デフォルトの許容誤差（0.1 mm = 1e-4 m）。
    pub const DEFAULT: Self = Self(1e-4);

    /// メートル単位の数値から許容誤差を生成する。
    ///
    /// 負数、NaN、無限大が指定された場合はエラーを返す。
    /// 0.0 は許容される。
    ///
    /// # 引数
    /// * `meters` - 許容誤差（メートル単位、0以上かつ有限数）
    ///
    /// # エラー
    /// `meters` が負数、NaN、または無限大の場合、[`GeometryError::ToleranceOutOfRange`] を返す。
    pub fn from_meters(meters: f64) -> Result<Self, Error> {
        if meters < 0.0 || !meters.is_finite() {
            Err(GeometryError::ToleranceOutOfRange { tolerance: meters }.into())
        } else {
            Ok(Self(meters))
        }
    }

    /// バリデーションを行わずに許容誤差を生成する。
    ///
    /// # Safety
    /// `meters` は 0 以上の有限数である必要がある。
    pub const unsafe fn from_meters_unchecked(meters: f64) -> Self {
        Self(meters)
    }

    /// メートル単位の許容誤差の数値を返す。
    #[inline]
    pub const fn as_meters(self) -> f64 {
        self.0
    }
}

impl Default for Tolerance {
    #[inline]
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl fmt::Display for Tolerance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} m", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_tolerance() {
        let t = Tolerance::from_meters(0.01).unwrap();
        assert_eq!(t.as_meters(), 0.01);

        let t_zero = Tolerance::from_meters(0.0).unwrap();
        assert_eq!(t_zero.as_meters(), 0.0);
    }

    #[test]
    fn default_tolerance() {
        assert_eq!(Tolerance::default(), Tolerance::DEFAULT);
        assert_eq!(Tolerance::DEFAULT.as_meters(), 1e-4);
    }

    #[test]
    fn invalid_tolerance() {
        assert!(Tolerance::from_meters(-0.001).is_err());
        assert!(Tolerance::from_meters(f64::NAN).is_err());
        assert!(Tolerance::from_meters(f64::INFINITY).is_err());
        assert!(Tolerance::from_meters(f64::NEG_INFINITY).is_err());
    }
}
