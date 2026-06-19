use core::ops::Sub;

use alloc::vec::Vec;

use super::{DiffuseParam, diffuse_along};
use crate::{Error, SpatialIdCollection, UnaryOperator};

/// 南北（Y）方向への波及を行う。Y方向は巡回せず、境界に当たった向きはクリップする。
pub struct YDiffuse;

impl<A: Ord + PartialEq + Clone + Sub<Output = A>> UnaryOperator<A> for YDiffuse {
    type CustomParameter = DiffuseParam<A>;
    type ResultValue = A;

    fn execution<S, O>(a: &S, custom_parameter: Self::CustomParameter) -> Result<O, Error>
    where
        S: SpatialIdCollection<Value = A>,
        O: SpatialIdCollection<Value = A>,
    {
        diffuse_along::<S, O, _>(a, custom_parameter, true, |flex_id, z, index| {
            Ok(flex_id.shift_y(z, index)?.collect::<Vec<_>>())
        })
    }
}
