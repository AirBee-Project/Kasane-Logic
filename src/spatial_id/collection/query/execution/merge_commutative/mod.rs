use super::Query;
use crate::SpatialIdCollection;
use crate::spatial_id::collection::query::traits::{UnaryOperator, WorkingTree};
use alloc::boxed::Box;
use alloc::vec::Vec;

impl<S: SpatialIdCollection> Query<S>
where
    S::Value: 'static,
{
    /// ② 可換グループ内で演算子をまとめてmergeし、最小個数にする。
    ///
    /// `Query::CommutativeGroup`（互いに可換であることが保証された区間）にだけ介入する。
    /// `Unary`/`Binary` は子を再帰するだけで通過させる（可換性が保証されていない区間では
    /// 演算子の並び替え・マージを行わない）。
    pub fn merge_commutative_ops(self) -> Self
    where
        S::Working: 'static,
    {
        match self {
            Query::CommutativeGroup(info, ops, input) => Query::CommutativeGroup(
                info,
                first_fit_merge(ops),
                Box::new(input.merge_commutative_ops()),
            ),
            Query::Unary(ops, input) => Query::Unary(ops, Box::new(input.merge_commutative_ops())),
            Query::Binary(op, lhs, rhs) => Query::Binary(
                op,
                Box::new(lhs.merge_commutative_ops()),
                Box::new(rhs.merge_commutative_ops()),
            ),
            other => other,
        }
    }
}

/// まだ確定していない集約候補（open accumulators）に対して先頭から順に `try_merge` を試し、
/// 最初に成功したものへ合成する（First-Fit）。どれとも合成できなければ新しい集約候補として
/// 追加する。
///
/// 具象型（`TypeId`）でのバケット化は行わない: `try_merge` は同型マージだけでなく
/// `ShiftX + ShiftY -> ShiftFXY` のような異なる具象型間のマージ（[`MergeAccumulator`]
/// 経由）も表現できるため、型で足切りすると発見できなくなる。`CommutativeGroup` は
/// そもそも同じ可換パターン（[`CommutativityInfo`]）を持つ演算子だけで構成されており
/// グループサイズも小さいため、素朴な総当たりで十分。
///
/// [`MergeAccumulator`]: crate::spatial_id::collection::query::traits::MergeAccumulator
/// [`CommutativityInfo`]: super::group_commutative::types::CommutativityInfo
fn first_fit_merge<W>(ops: Vec<Box<dyn UnaryOperator<W>>>) -> Vec<Box<dyn UnaryOperator<W>>>
where
    W: WorkingTree + 'static,
{
    let mut open: Vec<Box<dyn UnaryOperator<W>>> = Vec::new();
    for op in ops {
        let mut placed = false;
        for acc in open.iter_mut() {
            if let Some(merged) = acc.try_merge(op.as_ref()) {
                *acc = merged;
                placed = true;
                break;
            }
        }
        if !placed {
            open.push(op);
        }
    }
    open
}

#[cfg(test)]
mod test;
