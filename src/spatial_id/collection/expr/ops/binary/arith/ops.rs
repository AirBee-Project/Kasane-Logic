use core::ops::{Add as StdAdd, Mul as StdMul, Sub as StdSub};

use crate::SpatialIdCollection;

use super::add::Add;
use super::mul::Mul;
use super::sub::Sub;

use crate::spatial_id::collection::expr::query::Query;

impl<C: SpatialIdCollection> Query<C>
where
    C::Value: 'static + StdAdd<Output = C::Value>,
{
    #[allow(clippy::should_implement_trait)]
    pub fn add(self, other: impl Into<Query<C>>) -> Self {
        let kernel = alloc::boxed::Box::new(
            crate::spatial_id::collection::expr::ops::binary::BinaryOpKernel::<Add, _> {
                param: (),
                _op: core::marker::PhantomData,
            },
        );
        Query::Binary(
            crate::spatial_id::collection::expr::ops::binary::BinaryOp::Custom(kernel),
            alloc::boxed::Box::new(self),
            alloc::boxed::Box::new(other.into()),
        )
    }
}

impl<C: SpatialIdCollection> Query<C>
where
    C::Value: 'static + StdSub<Output = C::Value>,
{
    #[allow(clippy::should_implement_trait)]
    pub fn sub(self, other: impl Into<Query<C>>) -> Self {
        let kernel = alloc::boxed::Box::new(
            crate::spatial_id::collection::expr::ops::binary::BinaryOpKernel::<Sub, _> {
                param: (),
                _op: core::marker::PhantomData,
            },
        );
        Query::Binary(
            crate::spatial_id::collection::expr::ops::binary::BinaryOp::Custom(kernel),
            alloc::boxed::Box::new(self),
            alloc::boxed::Box::new(other.into()),
        )
    }
}

impl<C: SpatialIdCollection> Query<C>
where
    C::Value: 'static + StdMul<Output = C::Value>,
{
    #[allow(clippy::should_implement_trait)]
    pub fn mul(self, other: impl Into<Query<C>>) -> Self {
        let kernel = alloc::boxed::Box::new(
            crate::spatial_id::collection::expr::ops::binary::BinaryOpKernel::<Mul, _> {
                param: (),
                _op: core::marker::PhantomData,
            },
        );
        Query::Binary(
            crate::spatial_id::collection::expr::ops::binary::BinaryOp::Custom(kernel),
            alloc::boxed::Box::new(self),
            alloc::boxed::Box::new(other.into()),
        )
    }
}
