use crate::spatial_id::time::span::Span;
use crate::spatial_id::zoom_level::TZoomLevel;

/// 2の冪乗の長さを持ち、時間木の特定の境界にアライメントされた1ノード。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Segment {
    zoom: TZoomLevel,
    index: u64,
}

impl Segment {
    /// 新しい時間Segmentを生成する。
    pub const fn new(zoom: TZoomLevel, index: u64) -> Self {
        Self { zoom, index }
    }

    /// このSegmentのズームレベル（深さ）。
    pub const fn zoom(&self) -> TZoomLevel {
        self.zoom
    }

    /// このズームレベルにおけるインデックス。
    pub const fn index(&self) -> u64 {
        self.index
    }

    /// このSegmentが表す絶対秒区間 `[start, end)` を返す。
    ///
    /// 1Segmentの幅は `2^(TZoomLevel::MAX - zoom)` 秒。
    pub const fn span(&self) -> Span {
        let width = self.zoom.segment_seconds();
        let start = self.index * width;
        Span::new_unchecked(start, start + width)
    }
}

impl From<Segment> for Span {
    fn from(segment: Segment) -> Self {
        segment.span()
    }
}
