#[allow(unused_imports)]
use alloc::boxed::Box;
#[allow(unused_imports)]
use alloc::rc::Rc;
#[allow(unused_imports)]
use alloc::string::{String, ToString};
#[allow(unused_imports)]
use alloc::vec::Vec;

use crate::{Error, SpatialIdCollection, UnaryOperator};

use super::FillDefault;

pub trait FillOps: SpatialIdCollection {
    fn fill_default(&self, default: Self::Value) -> Result<Self, Error> {
        FillDefault::execution::<Self, Self>(self, default)
    }
}

impl<C> FillOps for C where C: SpatialIdCollection {}
