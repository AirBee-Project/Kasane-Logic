use super::Query;
use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::query::execution::group_commutative::types::CommutativityInfo;
use crate::spatial_id::collection::query::traits::UnaryOperator;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ops::Range;

pub mod types;

/// 単項演算子の並びの中の、**並べ替えてよい連続区間**。
pub(crate) struct CommutativeRun {
    /// 元の `ops` 内での範囲。
    pub(crate) range: Range<usize>,
    /// この区間の可換性（区間の先頭演算子のもの）。
    pub(crate) info: CommutativityInfo,
    /// 並べ替えてよいか。2個以上あり、かつ可換性を主張している区間だけが真。
    pub(crate) sortable: bool,
}

/// `ops` を可換区間へ切り分ける。
///
/// AST を組み替える [`Query::group_commutative_ops`] と、AST を借用したまま実行順だけを
/// 決める [`optimized_unary_order`] の**両方がこれを使う**。区間の定義が2箇所にあると、
/// AST を消費できる [`run`](Query::run) と借用しか持てない
/// [`run_within`](Query::run_within) で演算子の適用順が食い違う。
pub(crate) fn commutative_runs<V: SafeValue + 'static>(
    ops: &[Box<dyn UnaryOperator<V>>],
) -> Vec<CommutativeRun> {
    let mut runs: Vec<CommutativeRun> = Vec::new();
    let mut start = 0usize;
    let mut current_info: Option<CommutativityInfo> = None;

    let mut close = |range: Range<usize>, info: Option<CommutativityInfo>| {
        if range.is_empty() {
            return;
        }
        let info = info.expect("非空の区間には必ず先頭演算子の可換性がある");
        let sortable = range.len() > 1 && info.is_potentially_commutative();
        runs.push(CommutativeRun {
            range,
            info,
            sortable,
        });
    };

    for (i, op) in ops.iter().enumerate() {
        let info = op.commutativity_info();

        // 区間を伸ばせるのは、既存の全員と可換なとき（クリークを保つ）か、
        // 可換性を主張しない演算子が連続しているとき（並べ替えないまま束ねる）。
        let mut is_clique = false;
        if let Some(cur_info) = current_info
            && cur_info.is_potentially_commutative()
            && info.is_potentially_commutative()
        {
            is_clique = ops[start..i]
                .iter()
                .all(|existing| existing.commutativity_info().can_commute_with(&info));
        }
        let extends = is_clique
            || (current_info.is_some_and(|c| !c.is_potentially_commutative())
                && !info.is_potentially_commutative());

        if !extends {
            close(start..i, current_info);
            start = i;
            current_info = Some(info);
        }
    }
    close(start..ops.len(), current_info);

    runs
}

/// `ops` を借用したまま、最適化後の**実行順**へ並べ替えた参照列を返す。
///
/// [`Query::group_commutative_ops`] + [`Query::sort_commutative_ops`] を AST へ適用した
/// 結果と同じ順序になる。AST を消費できない経路（`&self` を取る
/// [`run_within`](Query::run_within) / [`lazy_get`](Query::lazy_get)）のための入口。
pub(crate) fn optimized_unary_order<V: SafeValue + 'static>(
    ops: &[Box<dyn UnaryOperator<V>>],
) -> Vec<&dyn UnaryOperator<V>> {
    let mut order: Vec<&dyn UnaryOperator<V>> = ops.iter().map(|op| &**op).collect();
    for run in commutative_runs(ops) {
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

impl<V: SafeValue + 'static> Query<V> {
    /// 可換な部分を検知して囲む
    ///
    /// ASTの `Query::Unary` 内に直列に並んだ演算子（`ops`）を走査し、
    /// 互いに可換な連続区間を見つけたら `CommutativeGroup` にラップします。
    pub fn group_commutative_ops(self) -> Self {
        match self {
            Query::Unary(ops, input) => {
                // 区間の切り分けは `commutative_runs` に一本化してある。並べ替え自体は
                // `sort_commutative_ops` が行うので、ここでは順序を保ったまま束ねるだけ。
                let runs = commutative_runs(&ops);
                let mut ast = input.group_commutative_ops();
                let mut remaining = ops.into_iter();

                for run in runs {
                    let group: Vec<Box<dyn UnaryOperator<V>>> =
                        remaining.by_ref().take(run.range.len()).collect();
                    ast = if run.sortable {
                        Query::CommutativeGroup(run.info, group, Box::new(ast))
                    } else {
                        Query::Unary(group, Box::new(ast))
                    };
                }
                ast
            }
            Query::CommutativeGroup(info, ops, input) => {
                Query::CommutativeGroup(info, ops, Box::new(input.group_commutative_ops()))
            }
            Query::Binary(op, lhs, rhs) => Query::Binary(
                op,
                Box::new(lhs.group_commutative_ops()),
                Box::new(rhs.group_commutative_ops()),
            ),
            other => other,
        }
    }
}

#[cfg(test)]
mod test;
