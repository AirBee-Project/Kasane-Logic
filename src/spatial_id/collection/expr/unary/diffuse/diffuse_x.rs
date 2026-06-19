use core::ops::Sub;

use alloc::vec::Vec;

use super::{DiffuseParam, diffuse_along};
use crate::{Error, SpatialIdCollection, UnaryOperator};

/// 東西（X）方向への波及を行う。X方向は地球を周回するため巡回する。
pub struct XDiffuse;

impl<A: Ord + PartialEq + Clone + Sub<Output = A>> UnaryOperator<A> for XDiffuse {
    type CustomParameter = DiffuseParam<A>;
    type ResultValue = A;

    fn execution<S, O>(a: &S, custom_parameter: Self::CustomParameter) -> Result<O, Error>
    where
        S: SpatialIdCollection<Value = A>,
        O: SpatialIdCollection<Value = A>,
    {
        // X方向は巡回し境界超過しないため、クリップは不要。
        diffuse_along::<S, O, _>(a, custom_parameter, false, |flex_id, z, index| {
            Ok(flex_id.shift_x(z, index)?.collect::<Vec<_>>())
        })
    }
}
