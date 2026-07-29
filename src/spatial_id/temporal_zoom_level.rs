use crate::{SpatialIdError, error::Error};
use core::fmt;

/// 時間軸の生の2進セルにおけるズームレベルを表す型。
///
/// 空間軸の[`ZoomLevel`](crate::ZoomLevel)と同じ思想だが、Fのような符号オフセットが無く、
/// 各ズームでのインデックス最大値も `(1u64 << z) - 1` と単純な式で求まるため、
/// 事前計算テーブルは持たない。
///
/// ```
/// # use kasane_logic::spatial_id::temporal_zoom_level::TZoomLevel;
/// let z = TZoomLevel::new(5).unwrap();
/// assert_eq!(z.get(), 5);
/// ```
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct TZoomLevel(u8);

impl fmt::Display for TZoomLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl TZoomLevel {
    /// 最小のズームレベル（`0`）。1セルが全期間（`2^62`秒）を表す。
    pub const MIN: TZoomLevel = TZoomLevel(0);

    /// 最大のズームレベル。1セルが1秒を表す。
    ///
    /// FlexTreeの`LEAF_LEVEL`センチネルが`u8`に収まるよう、63ではなく62を選んでいる
    /// （4軸・Tを最深軸とした場合 `4*(MAX+1)` が `u8::MAX` を超えないようにするため）。
    pub const MAX: TZoomLevel = TZoomLevel(62);

    /// `z` が `0..=`[`TZoomLevel::MAX`] の範囲内であることを検証して [`TZoomLevel`] を生成する。
    pub const fn new(z: u8) -> Result<Self, Error> {
        if z > Self::MAX.0 {
            return Err(Error::SpatialId(SpatialIdError::ZOutOfRange { z }));
        }
        Ok(TZoomLevel(z))
    }

    /// 検証を行わずに [`TZoomLevel`] を生成する。
    ///
    /// # Safety
    /// 呼び出し側は `z <= `[`TZoomLevel::MAX`] を保証しなければならない。
    pub const unsafe fn new_unchecked(z: u8) -> Self {
        TZoomLevel(z)
    }

    /// 1段深いズームレベルを返す。[`TZoomLevel::MAX`] なら [`None`]。
    pub const fn deeper(self) -> Option<TZoomLevel> {
        if self.0 >= Self::MAX.0 {
            None
        } else {
            Some(TZoomLevel(self.0 + 1))
        }
    }

    /// 保持しているズームレベルを `u8` として返す。
    pub const fn get(self) -> u8 {
        self.0
    }

    /// このズームレベルにおけるインデックスの最大値（`2^z - 1`）。
    pub const fn max_index(self) -> u64 {
        (1u64 << self.0) - 1
    }

    /// このズームレベルにおける1セルの秒数（`2^(MAX - z)`）。
    pub const fn cell_seconds(self) -> u64 {
        1u64 << (Self::MAX.0 - self.0)
    }

    /// `index` がこのズームレベルの範囲に収まるか検証する。
    pub const fn check_index(self, index: u64) -> Result<(), Error> {
        if index > self.max_index() {
            return Err(Error::SpatialId(SpatialIdError::TOutOfRange {
                i: self.cell_seconds(),
                t: index,
            }));
        }
        Ok(())
    }
}

impl TryFrom<u8> for TZoomLevel {
    type Error = Error;

    fn try_from(z: u8) -> Result<Self, Error> {
        TZoomLevel::new(z)
    }
}

impl From<TZoomLevel> for u8 {
    fn from(z: TZoomLevel) -> u8 {
        z.0
    }
}
