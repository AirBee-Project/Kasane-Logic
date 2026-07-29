#[cfg(not(feature = "temporal_id"))]
mod disabled;
#[cfg(not(feature = "temporal_id"))]
pub use disabled::{TemporalCell, TemporalRange};

#[cfg(feature = "temporal_id")]
use crate::{Side, error::Error, spatial_id::temporal_zoom_level::TZoomLevel};

#[cfg(feature = "temporal_id")]
pub mod impls;

pub mod interval;
pub mod range;
#[cfg(feature = "temporal_id")]
pub use range::TemporalRange;

#[cfg(feature = "temporal_id")]
#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
/// FlexTree内部が実際に木へ格納する、時間軸の生の2進セル表現。
///
/// 1セルは`2^(TZoomLevel::MAX - zoom)`秒（＝2の冪秒）の区間を表す。[`FlexId`](crate::FlexId)/
/// [`SingleId`](crate::SingleId)（点）が保持する。人間に読みやすい表現は[`TemporalRange`]
/// を参照。
pub struct TemporalCell {
    /// 時間軸のズームレベル。
    zoom: TZoomLevel,
    /// このズームレベルにおけるインデックス。
    index: u64,
}

#[cfg(feature = "temporal_id")]
impl TemporalCell {
    /// 全時間を表す定数（ズーム0・インデックス0、全期間`2^62`秒を1セルで覆う）。
    pub const WHOLE: Self = TemporalCell {
        zoom: TZoomLevel::MIN,
        index: 0,
    };

    /// このインスタンスが全時間を表す特別な値（[`WHOLE`](Self::WHOLE)）であるかを判定する。
    pub fn is_whole(&self) -> bool {
        self.zoom.get() == 0 && self.index == 0
    }

    /// 指定されたズームレベルとインデックスから新しい [`TemporalCell`] を構築する。
    pub fn new(zoom: u8, index: u64) -> Result<Self, Error> {
        let zoom = TZoomLevel::new(zoom)?;
        zoom.check_index(index)?;
        Ok(Self { zoom, index })
    }

    /// 検証を行わずに [`TemporalCell`] を構築する。
    ///
    /// # Safety
    /// 呼び出し側は `zoom <= TZoomLevel::MAX` かつ `index <= 2^zoom - 1` を保証しなければならない。
    pub unsafe fn new_unchecked(zoom: u8, index: u64) -> Self {
        Self {
            zoom: unsafe { TZoomLevel::new_unchecked(zoom) },
            index,
        }
    }

    /// 時間軸のズームレベルを取得する。
    pub fn zoom(&self) -> u8 {
        self.zoom.get()
    }

    /// このズームレベルにおけるインデックスを取得する。
    pub fn index(&self) -> u64 {
        self.index
    }

    /// この [`TemporalCell`] を2つに切り分ける。[`TZoomLevel::MAX`] なら [`None`]。
    pub fn split(&self, side: Side) -> Option<Self> {
        let zoom = self.zoom.deeper()?;
        let index = self.index * 2 + side as u64;
        Some(Self { zoom, index })
    }

    /// 2つの [`TemporalCell`] の重なっている区間（Intersection）を計算して返す。
    /// 重なりがない場合は `None` を返す。
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        let (deep, shallow) = if self.zoom.get() > other.zoom.get() {
            (self, other)
        } else {
            (other, self)
        };

        let shift = deep.zoom.get() - shallow.zoom.get();

        if (deep.index >> shift) == shallow.index {
            Some(deep.clone())
        } else {
            None
        }
    }

    /// 相手の [`TemporalCell`] との差集合（`self - other`）を計算し、イテレータとして返す。
    pub fn difference(&self, other: &Self) -> impl Iterator<Item = Self> {
        use alloc::vec::Vec;

        let mut results = Vec::new();

        let intersect = match self.intersection(other) {
            Some(i) => i,
            None => {
                results.push(self.clone());
                return results.into_iter();
            }
        };

        if self == &intersect {
            return results.into_iter();
        }

        let mut current = self.clone();

        while current.zoom.get() < intersect.zoom.get() {
            let lower = current.split(Side::Lower).unwrap();
            let upper = current.split(Side::Upper).unwrap();
            if lower.intersection(&intersect).is_some() {
                results.push(upper);
                current = lower;
            } else {
                results.push(lower);
                current = upper;
            }
        }

        results.into_iter()
    }
}

#[cfg(feature = "temporal_id")]
impl crate::spatial_id::traits::TemporalId for TemporalCell {
    const WHOLE: Self = Self::WHOLE;

    fn is_whole(&self) -> bool {
        Self::is_whole(self)
    }

    fn seconds_range(&self) -> (u64, u64) {
        let width = self.zoom.cell_seconds();
        let start = self.index * width;
        (start, start + width)
    }
}
