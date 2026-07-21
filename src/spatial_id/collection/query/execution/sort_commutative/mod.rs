use super::Query;
use crate::SpatialIdCollection;
use alloc::boxed::Box;

impl<S: SpatialIdCollection> Query<S>
where
    S::Value: 'static,
{
    /// ③ 可換グループ内の演算子を「拡大率（[`expansion_ratio`]）」が小さい順へ並び替える。
    ///
    /// [`group_commutative_ops`](Self::group_commutative_ops) と並列の**静的な AST 変換**で、
    /// `Query::CommutativeGroup`（互いに可換であることが保証された区間）にだけ介入する。
    /// `Unary`/`Binary` は子を再帰するだけで通過させる。
    ///
    /// グループ内の演算子は互いに可換なので、順序を入れ替えても結果は変わらない。拡大率の小さい
    /// （中間データをあまり増やさない）演算子を先に適用することで、連続する拡大操作の処理コスト
    /// 総和を抑えるためだけの並び替え。
    ///
    /// [`expansion_ratio`]: crate::spatial_id::collection::query::traits::UnaryOperator::expansion_ratio
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
