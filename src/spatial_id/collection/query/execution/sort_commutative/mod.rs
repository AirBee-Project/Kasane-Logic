use super::Query;
use crate::SpatialIdCollection;
use alloc::boxed::Box;

impl<S: SpatialIdCollection> Query<S>
where
    S::Value: 'static,
{
    /// 可換グループ内の演算子を「拡大率（[`expansion_ratio`]）」が小さい順へ並び替える。
    pub fn sort_commutative_ops(self) -> Self
    where
        S::Working: 'static,
    {
        match self {
            Query::CommutativeGroup(info, mut ops, input) => {
                ops.sort_by(|a, b| {
                    a.expansion_ratio()
                        .partial_cmp(&b.expansion_ratio())
                        .unwrap_or(core::cmp::Ordering::Equal)
                });
                Query::CommutativeGroup(info, ops, Box::new(input.sort_commutative_ops()))
            }
            Query::Unary(ops, input) => Query::Unary(ops, Box::new(input.sort_commutative_ops())),
            Query::Binary(op, lhs, rhs) => Query::Binary(
                op,
                Box::new(lhs.sort_commutative_ops()),
                Box::new(rhs.sort_commutative_ops()),
            ),
            other => other,
        }
    }
}
