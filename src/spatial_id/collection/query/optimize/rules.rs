use crate::SpatialIdCollection;
use crate::spatial_id::collection::query::Query;

/// 恒等な単項演算を取り除く。
pub(super) fn drop_identity_unary<C: SpatialIdCollection>(plan: Query<C>) -> Query<C>
where
    C::Value: 'static,
{
    match plan {
        Query::Unary(op, input) if op.is_identity() => *input,
        other => other,
    }
}
