#[cfg(not(feature = "temporal_id"))]
mod disabled;
#[cfg(not(feature = "temporal_id"))]
pub use disabled::TemporalId;
#[cfg(not(feature = "temporal_id"))]
pub(crate) use disabled::TemporalSegment;

#[cfg(feature = "temporal_id")]
use crate::{Side, error::Error, spatial_id::temporal_id::zoom_level::TZoomLevel};

#[cfg(feature = "temporal_id")]
pub mod impls;

pub mod interval;
pub mod range;
pub mod zoom_level;
#[cfg(feature = "temporal_id")]
pub use range::TemporalId;

#[cfg(feature = "temporal_id")]
#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
/// FlexTree内部が実際に木へ格納する、時間軸の生の2進セル表現。クレート内部の実装詳細であり
/// 公開APIには出てこない（[`FlexId`](crate::FlexId)/[`SingleId`](crate::SingleId)の
/// プライベートフィールドとしてのみ使う）。
///
/// 1セルは`2^(TZoomLevel::MAX - zoom)`秒（＝2の冪秒）の区間を表す。人間に読みやすい表現は
/// [`TemporalId`] を参照。
pub(crate) struct TemporalSegment {
    /// 時間軸のズームレベル。
    zoom: TZoomLevel,
    /// このズームレベルにおけるインデックス。
    index: u64,
}

#[cfg(feature = "temporal_id")]
impl TemporalSegment {
    /// 全時間を表す定数（ズーム0・インデックス0、全期間`2^62`秒を1セルで覆う）。
    pub const WHOLE: Self = TemporalSegment {
        zoom: TZoomLevel::MIN,
        index: 0,
    };

    /// このインスタンスが全時間を表す特別な値（[`WHOLE`](Self::WHOLE)）であるかを判定する。
    pub fn is_whole(&self) -> bool {
        self.zoom.get() == 0 && self.index == 0
    }

    /// 指定されたズームレベルとインデックスから新しい [`TemporalSegment`] を構築する。
    pub fn new(zoom: u8, index: u64) -> Result<Self, Error> {
        let zoom = TZoomLevel::new(zoom)?;
        zoom.check_index(index)?;
        Ok(Self { zoom, index })
    }

    /// 検証を行わずに [`TemporalSegment`] を構築する。
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

    /// この [`TemporalSegment`] を2つに切り分ける。[`TZoomLevel::MAX`] なら [`None`]。
    pub fn split(&self, side: Side) -> Option<Self> {
        let zoom = self.zoom.deeper()?;
        let index = self.index * 2 + side as u64;
        Some(Self { zoom, index })
    }

    /// 2つの [`TemporalSegment`] の重なっている区間（Intersection）を計算して返す。
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

    /// 相手の [`TemporalSegment`] との差集合（`self - other`）を計算し、イテレータとして返す。
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

    /// この [`TemporalSegment`] が表す絶対秒区間 `[start, end)` を返す。
    pub fn seconds_range(&self) -> (u64, u64) {
        let width = self.zoom.cell_seconds();
        let start = self.index * width;
        (start, start + width)
    }
}
