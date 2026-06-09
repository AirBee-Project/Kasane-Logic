use crate::{Error, SpatialIdCollection, UnaryOperator};

use super::FillDefault;

pub trait FillOps: SpatialIdCollection {
    fn fill_default(&self, default: Self::Value) -> Result<Self, Error> {
        FillDefault::execution::<Self, Self>(self, default)
    }
}

impl<C> FillOps for C where C: SpatialIdCollection {}
