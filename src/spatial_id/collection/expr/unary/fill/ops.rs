use crate::{Error, SpatialIdCollection, UnaryOperator};

use super::FillDefault;

pub trait FillOps: SpatialIdCollection {
    fn fill_default(&self, default: Self::Value) -> Result<Self, Error> {
        FillDefault::execution::<Self, Self>(self, default)
    }
}

impl<C> FillOps for C where C: SpatialIdCollection {}

use crate::spatial_id::collection::expr::plan::Plan;

impl<C: SpatialIdCollection> Plan<C>
where
    C::Value: 'static,
{
    pub fn fill_default(self, default: C::Value) -> Self {
        Plan::Unary(
            crate::spatial_id::collection::expr::plan::UnaryOp::Fill(default),
            alloc::boxed::Box::new(self),
        )
    }
}
