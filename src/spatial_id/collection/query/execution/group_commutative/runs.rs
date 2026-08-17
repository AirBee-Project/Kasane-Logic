use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::query::execution::group_commutative::types::CommutativityInfo;
use crate::spatial_id::collection::query::traits::UnaryOperator;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ops::Range;

/// 単項演算子の並びの中の、**並べ替えてよい連続区間**。
pub(crate) struct CommutativeRun {
    /// 元の `ops` 内での範囲。
    pub(crate) range: Range<usize>,
    /// この区間の可換性（区間の先頭演算子のもの）。
    pub(crate) info: CommutativityInfo,
    /// 並べ替えてよいか。2個以上あり、かつ可換性を主張している区間だけが真。
    pub(crate) sortable: bool,
}

pub(crate) trait UnaryOperatorSliceExt<V> {
    /// `ops` を可換区間へ切り分ける。
    fn commutative_runs(&self) -> Vec<CommutativeRun>;

    /// 借用したまま、最適化後の**実行順**へ並べ替えた参照列を返す。
    fn optimized_order(&self) -> Vec<&dyn UnaryOperator<V>>;
}

impl<V: SafeValue + 'static> UnaryOperatorSliceExt<V> for [Box<dyn UnaryOperator<V>>] {
    fn commutative_runs(&self) -> Vec<CommutativeRun> {
        if self.is_empty() {
            return Vec::new();
        }

        let mut runs = Vec::new();
        let mut start = 0;
        let mut current_info = self[0].commutativity_info();

        for (i, op) in self.iter().enumerate().skip(1) {
            let info = op.commutativity_info();

            let mut extends = false;
            if current_info.is_potentially_commutative() && info.is_potentially_commutative() {
                // 区間を伸ばせるのは、既存の全員と可換なとき（クリークを保つ）
                extends = self[start..i]
                    .iter()
                    .all(|existing| existing.commutativity_info().can_commute_with(&info));
            } else if !current_info.is_potentially_commutative()
                && !info.is_potentially_commutative()
            {
                // 可換性を主張しない演算子が連続しているとき（並べ替えないまま束ねる）
                extends = true;
            }

            if !extends {
                let sortable = (i - start) > 1 && current_info.is_potentially_commutative();
                runs.push(CommutativeRun {
                    range: start..i,
                    info: current_info,
                    sortable,
                });
                start = i;
                current_info = info;
            }
        }

        let sortable = (self.len() - start) > 1 && current_info.is_potentially_commutative();
        runs.push(CommutativeRun {
            range: start..self.len(),
            info: current_info,
            sortable,
        });

        runs
    }

    fn optimized_order(&self) -> Vec<&dyn UnaryOperator<V>> {
        let mut order: Vec<&dyn UnaryOperator<V>> = self.iter().map(|op| &**op).collect();
        for run in self.commutative_runs() {
            if run.sortable {
                order[run.range].sort_by(|a, b| {
                    a.expansion_ratio()
                        .partial_cmp(&b.expansion_ratio())
                        .unwrap_or(core::cmp::Ordering::Equal)
                });
            }
        }
        order
    }
}
