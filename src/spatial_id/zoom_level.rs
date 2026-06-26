use crate::{SpatialIdError, error::Error};
use core::fmt;

/// ズームレベルを表す型。
/// ```
/// # use kasane_logic::{SpatialIdError, ZoomLevel};
/// let z = ZoomLevel::new(5).unwrap();
/// assert_eq!(z.get(), 5);
/// assert_eq!(ZoomLevel::new(255), Err(SpatialIdError::ZOutOfRange { z: 255 }.into()));
/// ```
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ZoomLevel(u8);

impl fmt::Display for ZoomLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl ZoomLevel {
    /// 各ズームレベルにおけるXYインデックスの最大値
    const XY_MAX: [u32; Self::MAX.0 as usize + 1] = [
        0, 1, 3, 7, 15, 31, 63, 127, 255, 511, 1023, 2047, 4095, 8191, 16383, 32767, 65535, 131071,
        262143, 524287, 1048575, 2097151, 4194303, 8388607, 16777215, 33554431, 67108863,
        134217727, 268435455, 536870911, 1073741823,
    ];

    /// 各ズームレベルにおけるFインデックスの最小値
    const F_MIN: [i32; Self::MAX.0 as usize + 1] = [
        -1,
        -2,
        -4,
        -8,
        -16,
        -32,
        -64,
        -128,
        -256,
        -512,
        -1024,
        -2048,
        -4096,
        -8192,
        -16384,
        -32768,
        -65536,
        -131072,
        -262144,
        -524288,
        -1048576,
        -2097152,
        -4194304,
        -8388608,
        -16777216,
        -33554432,
        -67108864,
        -134217728,
        -268435456,
        -536870912,
        -1073741824,
    ];

    /// 各ズームレベルにおけるFインデックスの最大値
    const F_MAX: [i32; Self::MAX.0 as usize + 1] = [
        0, 1, 3, 7, 15, 31, 63, 127, 255, 511, 1023, 2047, 4095, 8191, 16383, 32767, 65535, 131071,
        262143, 524287, 1048575, 2097151, 4194303, 8388607, 16777215, 33554431, 67108863,
        134217727, 268435455, 536870911, 1073741823,
    ];

    /// 最小のズームレベル（`0`）。
    pub const MIN: ZoomLevel = ZoomLevel(0);

    /// 最大のズームレベル。
    pub const MAX: ZoomLevel = ZoomLevel(30);

    /// `z` が `0..=`[`ZoomLevel::MAX`] の範囲内であることを検証して [`ZoomLevel`] を生成する。
    ///
    /// # バリデーション
    /// - `z` が [`ZoomLevel::MAX`] を超える場合は [`SpatialIdError::ZOutOfRange`] を返す。
    pub const fn new(z: u8) -> Result<Self, Error> {
        if z > Self::MAX.0 {
            return Err(Error::SpatialId(SpatialIdError::ZOutOfRange { z }));
        }
        Ok(ZoomLevel(z))
    }

    /// 検証を行わずに [`ZoomLevel`] を生成する。
    ///
    /// # Safety
    /// 呼び出し側は `z <= `[`ZoomLevel::MAX`] を保証しなければならない。これを破ると、
    /// [`f_min`](Self::f_min) などの配列アクセスでパニックや未定義動作を引き起こす可能性がある。
    pub const unsafe fn new_unchecked(z: u8) -> Self {
        ZoomLevel(z)
    }

    /// 保持しているズームレベルを `u8` として返す。
    pub const fn get(self) -> u8 {
        self.0
    }

    /// このズームレベルにおける F インデックスの最小値（`unsafe { ZoomLevel::new_unchecked(z as u8) }.f_min()`）。
    pub const fn f_min(self) -> i32 {
        Self::F_MIN[self.0 as usize]
    }

    /// このズームレベルにおける F インデックスの最大値（`unsafe { ZoomLevel::new_unchecked(z as u8) }.f_max()`）。
    pub const fn f_max(self) -> i32 {
        Self::F_MAX[self.0 as usize]
    }

    /// このズームレベルにおける X / Y インデックスの最大値（`unsafe { ZoomLevel::new_unchecked(z as u8) }.xy_max()`）。
    pub const fn xy_max(self) -> u32 {
        Self::XY_MAX[self.0 as usize]
    }

    /// `f` がこのズームレベルの F 範囲に収まるか検証する。
    ///
    /// # バリデーション
    /// - 範囲外の場合は [`SpatialIdError::FOutOfRange`] を返す。
    pub const fn check_f(self, f: i32) -> Result<(), Error> {
        if f < self.f_min() || f > self.f_max() {
            return Err(Error::SpatialId(SpatialIdError::FOutOfRange {
                z: self.0,
                f,
            }));
        }
        Ok(())
    }

    /// `x` がこのズームレベルの X 範囲に収まるか検証する。
    ///
    /// # バリデーション
    /// - 範囲外の場合は [`SpatialIdError::XOutOfRange`] を返す。
    pub const fn check_x(self, x: u32) -> Result<(), Error> {
        if x > self.xy_max() {
            return Err(Error::SpatialId(SpatialIdError::XOutOfRange {
                z: self.0,
                x,
            }));
        }
        Ok(())
    }

    /// `y` がこのズームレベルの Y 範囲に収まるか検証する。
    ///
    /// # バリデーション
    /// - 範囲外の場合は [`SpatialIdError::YOutOfRange`] を返す。
    pub const fn check_y(self, y: u32) -> Result<(), Error> {
        if y > self.xy_max() {
            return Err(Error::SpatialId(SpatialIdError::YOutOfRange {
                z: self.0,
                y,
            }));
        }
        Ok(())
    }
}

impl TryFrom<u8> for ZoomLevel {
    type Error = Error;

    fn try_from(z: u8) -> Result<Self, Error> {
        ZoomLevel::new(z)
    }
}

impl From<ZoomLevel> for u8 {
    fn from(z: ZoomLevel) -> u8 {
        z.0
    }
}

/// `u8` もしくは `ZoomLevel` のどちらからでも `ZoomLevel` を抽出・生成するためのトレイト。
/// API の引数として `impl IntoZoomLevel` を受け取ることで、利用者は `4` などの数値と、
/// すでに検証済みの `ZoomLevel::MAX` の両方をシームレスに渡すことができる。
pub trait IntoZoomLevel {
    fn into_zoom_level(self) -> Result<ZoomLevel, Error>;
}

impl IntoZoomLevel for u8 {
    fn into_zoom_level(self) -> Result<ZoomLevel, Error> {
        ZoomLevel::new(self)
    }
}

impl IntoZoomLevel for ZoomLevel {
    fn into_zoom_level(self) -> Result<ZoomLevel, Error> {
        Ok(self)
    }
}
