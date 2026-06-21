use alloc::boxed::Box;
use core::marker::PhantomData;

use crate::{Error, SpatialIdCollection, UnaryOperator};

pub trait UnaryKernel<C: SpatialIdCollection> {
    fn run(self: Box<Self>, input: &C) -> Result<C, Error>;

    fn is_identity(&self) -> bool {
        false
    }
}

pub struct UnaryOpKernel<Op, P> {
    pub param: P,
    pub _op: PhantomData<fn() -> Op>,
}

impl<C, Op> UnaryKernel<C> for UnaryOpKernel<Op, <Op as UnaryOperator<C::Value>>::CustomParameter>
where
    C: SpatialIdCollection,
    Op: UnaryOperator<C::Value, ResultValue = C::Value>,
{
    fn run(self: Box<Self>, input: &C) -> Result<C, Error> {
        <Op as UnaryOperator<C::Value>>::execution::<C, C>(input, self.param)
    }

    fn is_identity(&self) -> bool {
        <Op as UnaryOperator<C::Value>>::is_identity(&self.param)
    }
}
