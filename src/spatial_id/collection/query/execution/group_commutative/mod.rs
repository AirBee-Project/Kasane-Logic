use super::Query;
use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::query::execution::group_commutative::types::CommutativityInfo;
use crate::spatial_id::collection::query::traits::UnaryOperator;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ops::Range;

pub mod types;

/// `ops` を可換な連続区間（クリーク）へ分割する。
///
/// 各要素は先頭から順に走査され、直前のクリークと可換であればそこへ加わり、
/// 不可換になった時点で新しいクリークを開始する（`Other` 同士は可換とはみなさない
/// が、隣接していればまとめて1つの区間として扱う）。返る `Range` は常に連続かつ
/// 元の並び順を保ったまま `0..ops.len()` を過不足なく覆う。
///
/// [`Query::group_commutative_ops`] と [`plan_order`] はどちらも「どこがクリークか」
/// という同じ判断を必要とするため、判定ロジックをここに一本化する。前者は所有権を
/// 消費してASTを組み替え、後者は `&self` のまま並び順（インデックス列）だけを返す
/// という使い道の違いだけがある。
fn partition_cliques<V: SafeValue + 'static>(
    ops: &[Box<dyn UnaryOperator<V>>],
) -> Vec<(CommutativityInfo, Range<usize>)> {
    let mut cliques = Vec::new();
    let Some(first) = ops.first() else {
        return cliques;
    };

    let mut start = 0;
    let mut clique_info = first.commutativity_info();

    for i in 1..ops.len() {
        let info = ops[i].commutativity_info();

        let is_clique = clique_info.is_potentially_commutative()
            && info.is_potentially_commutative()
            && ops[start..i]
                .iter()
                .all(|op| op.commutativity_info().can_commute_with(&info));

        let both_other =
            !clique_info.is_potentially_commutative() && !info.is_potentially_commutative();

        if !is_clique && !both_other {
            cliques.push((clique_info, start..i));
            start = i;
            clique_info = info;
        }
    }
    cliques.push((clique_info, start..ops.len()));
    cliques
}

impl<V: SafeValue + 'static> Query<V> {
    /// 可換な部分を検知して囲む
    ///
    /// ASTの `Query::Unary` 内に直列に並んだ演算子（`ops`）を走査し、
    /// 互いに可換な連続区間を見つけたら `CommutativeGroup` にラップします。
    pub fn group_commutative_ops(self) -> Self {
        match self {
            Query::Unary(mut ops, input) => {
                let mut current_ast = input.group_commutative_ops();

                for (info, range) in partition_cliques(&ops) {
                    let group: Vec<_> = ops.drain(..range.len()).collect();
                    current_ast = if group.len() > 1 && info.is_potentially_commutative() {
                        Query::CommutativeGroup(info, group, Box::new(current_ast))
                    } else {
                        Query::Unary(group, Box::new(current_ast))
                    };
                }
                current_ast
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

/// [`group_commutative_ops`](Query::group_commutative_ops) +
/// [`sort_commutative_ops`](super::Query::sort_commutative_ops) と同じ規則で導かれる実行順を、
/// `ops` の所有権を消費せずインデックス列として返す。
///
/// `Query::Unary`/`Query::CommutativeGroup` は所有権を移さないと組み替えられない
/// （`Box<dyn UnaryOperator>` は `Clone` できない）ため、`&Query` しか持てない
/// [`Query::run_on_subset`](super::Query::run_on_subset) 系はこれまで最適化を経由できなかった。
/// クリークの検出自体は [`partition_cliques`] を共有しており、ここでは各クリークを
/// `expansion_ratio` の昇順に並べ替えるだけでよい。
///
/// `Query::CommutativeGroup` の（全要素が互いに可換という不変条件を持つ）`ops` を渡しても
/// 正しく動く：全ペアが可換なら、このアルゴリズムは自然に1つのクリークとして検出し
/// `expansion_ratio` でソートするだけになる（＝ [`sort_commutative_ops`](super::Query::sort_commutative_ops) と同じ結果）。
/// そのため呼び出し側は `Unary` と `CommutativeGroup` を区別せずこの関数を使ってよい。
pub(crate) fn plan_order<V: SafeValue + 'static>(ops: &[Box<dyn UnaryOperator<V>>]) -> Vec<usize> {
    let mut order = Vec::with_capacity(ops.len());
    for (info, range) in partition_cliques(ops) {
        if range.len() > 1 && info.is_potentially_commutative() {
            let mut indices: Vec<usize> = range.collect();
            indices.sort_by(|&a, &b| {
                ops[a]
                    .expansion_ratio()
                    .partial_cmp(&ops[b].expansion_ratio())
                    .unwrap_or(core::cmp::Ordering::Equal)
            });
            order.extend(indices);
        } else {
            order.extend(range);
        }
    }
    order
}

#[cfg(test)]
mod test;
