//! シャード1枚の要約（[`ShardSummary`]）。
//!
//! 木を走査せずに「そのシャードを読む価値があるか」を判定するための、小さな値の束である。
//! サーバー側では KVS のシャード本体とは別キーへ置き、`read_subset` の入力領域と
//! 突き合わせて**本体を fetch する前に**枝刈りするために使う。
//!
//! # 何をここへ置き、何を木のノードへ置くか
//!
//! 木の内部ノード（`Node::Branch`）が持つキャッシュ
//! （`leaf_count` / `max_zoom` / `split_mask`）は**すべて位置非依存**である。これは
//! 偶然ではなく、`Node::collapse_equal_children` が「兄弟が等価なら畳む」正規化を行い、
//! `Node` の `PartialEq` が derive（＝全フィールド比較）であることの帰結である。
//! 兄弟は定義上その軸で位置が異なるので、
//! **位置に依存する値を `Node` に持たせると畳み込みが一度も発火せず、異方圧縮が丸ごと死ぬ**。
//! `Node::merge` の `ptr_eq` による構造共有も同時に失われる。
//!
//! そのため bounding box や絶対秒区間のような位置依存の要約は、ノードではなく
//! **木全体（＝シャード1枚）につき1つ**このモジュールが持つ。計算は O(葉数) だが、
//! シャードを直列化する経路は元々全ノードを走査するので、実質的な追加コストは無い。

use crate::spatial_id::collection::flex_tree::core::node::{Axis, axis_bit};
use crate::spatial_id::flex_id::ops::overlaps_axis;
use crate::{RangeId, SpatialId};

/// シャード1枚を、木を走査せずに評価するための要約。
///
/// [`SpatialIdSet::summary`](crate::SpatialIdSet::summary) /
/// [`SpatialIdMap::summary`](crate::SpatialIdMap::summary) /
/// [`SpatialIdTable::summary`](crate::SpatialIdTable::summary) で作る。
/// `persist` feature を有効にすると `SpatialIdMap::to_bytes` がこれをバイト列へ埋め込み、
/// `ArchivedSpatialIdMap::summary` が木を復元せずに読み出せる。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShardSummary {
    pub(crate) leaf_count: u32,
    pub(crate) split_mask: u8,
    /// F / X / Y それぞれの最大ズームレベル。空なら `[0, 0, 0]`。
    pub(crate) max_zoom: [u8; 3],
    /// F / X / Y それぞれの最小ズームレベル。空なら `[0, 0, 0]`。
    pub(crate) min_zoom: [u8; 3],
    /// 時間軸ズームの `[最小, 最大]`。空なら `[0, 0]`。
    pub(crate) t_zoom: [u8; 2],
    /// 値を持つSegment全体を包む最小の [`RangeId`]。空なら [`None`]。
    pub(crate) bbox: Option<RangeId>,
    /// 値を持つSegment全体が占める絶対秒区間 `[start, end)`。空なら [`None`]。
    ///
    /// 全Segmentが全時間なら `Some((0, 2^35))` になる（[`None`] は空だけを意味する）。
    pub(crate) seconds_range: Option<(u64, u64)>,
}

impl ShardSummary {
    /// 空のシャードの要約。
    pub(crate) fn empty() -> Self {
        Self {
            leaf_count: 0,
            split_mask: 0,
            max_zoom: [0; 3],
            min_zoom: [0; 3],
            t_zoom: [0; 2],
            bbox: None,
            seconds_range: None,
        }
    }

    /// 保持している [`FlexId`](crate::FlexId) の個数。
    pub fn count(&self) -> usize {
        self.leaf_count as usize
    }

    /// Segmentを1つも持たないか。
    pub fn is_empty(&self) -> bool {
        self.leaf_count == 0
    }

    /// 値を持つSegment全体を包む最小の [`RangeId`]。空なら [`None`]。
    ///
    /// シャード領域（[`SpatialIdSet::shard`](crate::SpatialIdSet::shard)）が
    /// 「割り当てられた広さ」なのに対し、こちらは「実際に埋まっている広さ」である。
    /// 疎なシャードでは後者が遥かに狭いので、枝刈りにはこちらを使う。
    pub fn bbox(&self) -> Option<&RangeId> {
        self.bbox.as_ref()
    }

    /// 値を持つSegment全体が占める絶対秒区間 `[start, end)`。空なら [`None`]。
    pub fn seconds_range(&self) -> Option<(u64, u64)> {
        self.seconds_range
    }

    /// F / X / Y それぞれの最大ズームレベル。
    pub fn max_zoom(&self) -> [u8; 3] {
        self.max_zoom
    }

    /// F / X / Y それぞれの最小ズームレベル。
    pub fn min_zoom(&self) -> [u8; 3] {
        self.min_zoom
    }

    /// 時間軸ズームの `[最小, 最大]`。
    pub fn t_zoom(&self) -> [u8; 2] {
        self.t_zoom
    }

    /// 全Segmentのズームが3軸とも揃っているか。
    ///
    /// 真なら、一様グリッドへの平坦化
    /// （`UniformGrid::from_tree`）が細分を伴わずに済む。
    pub fn is_uniform_zoom(&self) -> bool {
        !self.is_empty() && self.min_zoom == self.max_zoom
    }

    /// この木が時間軸（T）で分割されたノードを1つでも持つか。
    ///
    /// 偽なら全Segmentが全時間なので、書き出し時の時間方向の結合を丸ごと省ける。
    pub fn has_temporal_split(&self) -> bool {
        self.split_mask & axis_bit(Axis::T) != 0
    }

    /// 指定した領域と少しでも重なりうるか。**偽なら本体を読む必要が無い。**
    ///
    /// [`bbox`](Self::bbox) と `target` の空間的な重なり、および
    /// [`seconds_range`](Self::seconds_range) と `target` の時間的な重なりの両方を見る。
    /// 空のシャードは常に偽。
    ///
    /// bbox は保持Segmentの外接直方体なので、真でも実際には1つも交差しないことがある
    /// （偽陽性はあるが偽陰性は無い＝枝刈りとして安全）。
    pub fn intersects(&self, target: &RangeId) -> bool {
        let Some(bbox) = &self.bbox else {
            return false;
        };

        // 軸ごとの重なり判定は [`FlexId::intersects_range`] と同じ `overlaps_axis` を使う。
        // 枝刈りの判定と実際の交差判定が別実装だと、片方だけ直して食い違う。
        let axes = [
            (bbox.f().map(i64::from), target.f().map(i64::from)),
            (bbox.x().map(i64::from), target.x().map(i64::from)),
            (bbox.y().map(i64::from), target.y().map(i64::from)),
        ];

        axes.iter()
            .all(|(a, b)| overlaps_axis(bbox.z(), a[0], a[1], target.z(), b[0], b[1]))
            && self.intersects_seconds(target.seconds_range())
    }

    /// 指定した絶対秒区間 `[start, end)` と時間的に重なりうるか。空のシャードは常に偽。
    pub fn intersects_seconds(&self, target: (u64, u64)) -> bool {
        match self.seconds_range {
            Some((start, end)) => start < target.1 && target.0 < end,
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{RangeId, SingleId, SpatialIdSet};

    /// 要約の件数・ズームが実際の内容と一致すること。
    #[test]
    fn summary_reports_counts_and_zooms() {
        let mut set = SpatialIdSet::new();
        set.insert(SingleId::new(4, 0, 0, 0).unwrap());
        set.insert(SingleId::new(7, 2, 40, 40).unwrap());

        let summary = set.summary();
        assert_eq!(summary.count(), set.count());
        assert!(!summary.is_empty());
        assert_eq!(summary.max_zoom(), [7, 7, 7]);
        assert_eq!(summary.min_zoom(), [4, 4, 4]);
        assert!(
            !summary.is_uniform_zoom(),
            "ズームが揃っていないのに一様と報告された"
        );
    }

    /// 全Segmentのズームが揃っていれば一様と報告すること（グリッド経路の可否判定）。
    #[test]
    fn uniform_zoom_is_detected() {
        let mut set = SpatialIdSet::new();
        set.insert(SingleId::new(5, 1, 1, 1).unwrap());
        set.insert(SingleId::new(5, 1, 2, 1).unwrap());

        let summary = set.summary();
        assert!(summary.is_uniform_zoom());
        assert_eq!(summary.min_zoom(), summary.max_zoom());
    }

    /// 離れた領域は確実に枝刈りされること（偽陽性はあってよいが、ここは明確に外れている）。
    #[test]
    fn distant_targets_are_pruned() {
        let mut set = SpatialIdSet::new();
        set.insert(SingleId::new(5, 1, 1, 1).unwrap());
        let summary = set.summary();

        assert!(summary.intersects(&RangeId::new(5, [1, 1], [1, 1], [1, 1]).unwrap()));
        assert!(!summary.intersects(&RangeId::new(5, [1, 1], [20, 25], [1, 1]).unwrap()));
        assert!(!summary.intersects(&RangeId::new(5, [1, 1], [1, 1], [20, 25]).unwrap()));
    }

    /// 時間軸を使っていない木は「時間分割なし」と報告し、全時間を占めること。
    #[test]
    fn whole_time_tree_reports_no_temporal_split() {
        let mut set = SpatialIdSet::new();
        set.insert(SingleId::new(5, 1, 1, 1).unwrap());

        let summary = set.summary();
        assert!(!summary.has_temporal_split());
        assert_eq!(summary.t_zoom(), [0, 0]);
        let (start, end) = summary.seconds_range().unwrap();
        assert_eq!(start, 0);
        assert!(end > 0, "全時間の終端が 0 になっている");
    }

    /// 時間を持つSegmentを入れると、時間分割と占有秒区間が要約に出ること。
    #[cfg(feature = "temporal_id")]
    #[test]
    fn temporal_tree_reports_its_seconds_range() {
        use crate::{SpatialId, TZoomLevel};

        let id = SingleId::new(5, 1, 1, 1)
            .unwrap()
            .with_time_at(TZoomLevel::MAX.segment_seconds(), 1_770_000_000)
            .unwrap();
        let expected = id.seconds_range();

        let mut set = SpatialIdSet::new();
        set.insert(id);

        let summary = set.summary();
        assert!(summary.has_temporal_split());
        assert_eq!(summary.seconds_range(), Some(expected));

        // 重ならない時間窓は枝刈りされる。
        assert!(!summary.intersects_seconds((0, expected.0)));
        assert!(summary.intersects_seconds(expected));
    }
}
