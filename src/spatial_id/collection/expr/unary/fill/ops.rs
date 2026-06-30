use crate::SpatialIdCollection;

use crate::spatial_id::collection::expr::query::Query;

impl<C: SpatialIdCollection> Query<C>
where
    C::Value: 'static,
{
    pub fn fill_default(self, default: C::Value) -> Self {
        Query::Unary(
            crate::spatial_id::collection::expr::query::UnaryOp::Fill(default),
            alloc::boxed::Box::new(self),
        )
    }
}
