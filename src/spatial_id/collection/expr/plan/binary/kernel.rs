use alloc::boxed::Box;
use core::marker::PhantomData;

use crate::{BinaryOperator, Error, SpatialIdCollection};

pub trait BinaryKernel<C: SpatialIdCollection> {
    fn run(self: Box<Self>, lhs: &C, rhs: &C) -> Result<C, Error>;

    fn is_commutative(&self) -> bool {
        false
    }
}

pub struct BinaryOpKernel<Op, P> {
    pub param: P,
    pub _op: PhantomData<fn() -> Op>,
}

impl<C, Op> BinaryKernel<C>
    for BinaryOpKernel<Op, <Op as BinaryOperator<C::Value, C::Value>>::CustomParameter>
where
    C: SpatialIdCollection,
    Op: BinaryOperator<C::Value, C::Value, ResultValue = C::Value>,
{
    fn run(self: Box<Self>, lhs: &C, rhs: &C) -> Result<C, Error> {
        <Op as BinaryOperator<C::Value, C::Value>>::execution::<C, C, C>(lhs, rhs, self.param)
    }

    fn is_commutative(&self) -> bool {
        <Op as BinaryOperator<C::Value, C::Value>>::is_commutative(&self.param)
    }
}
