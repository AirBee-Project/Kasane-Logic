use super::{
    falloff_linear_f::FalloffLinearF, falloff_linear_x::FalloffLinearX,
    falloff_linear_y::FalloffLinearY,
};
use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::query::{execution::Query, merge_policy::MergePolicy};
use core::convert::TryFrom;
use core::fmt::Debug;
use core::ops::{Div, Mul, Sub};

impl<V: SafeValue + 'static> Query<V> {
    /// F方向のへ値をリニアに減少させる。
    /// 指定した距離で0になる。
    pub fn falloff_linear_f<Z: Into<u8>, P>(self, z: Z, radius: u32, _policy: P) -> Self
    where
        P: MergePolicy<V> + Send + Sync,
        V: Mul<Output = V> + Div<Output = V> + Sub<Output = V> + TryFrom<u32> + Clone + Send + Sync,
        <V as TryFrom<u32>>::Error: Debug,
    {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        match FalloffLinearF::<P>::new(z, radius) {
            Ok(op) => self.wrap_unary(op),
            Err(e) => Query::Error(e),
        }
    }

    /// X方向のへ値をリニアに減少させる。
    /// 指定した距離で0になる。
    pub fn falloff_linear_x<Z: Into<u8>, P>(self, z: Z, radius: u32, _policy: P) -> Self
    where
        P: MergePolicy<V> + Send + Sync,
        V: Mul<Output = V> + Div<Output = V> + Sub<Output = V> + TryFrom<u32> + Clone + Send + Sync,
        <V as TryFrom<u32>>::Error: Debug,
    {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        match FalloffLinearX::<P>::new(z, radius) {
            Ok(op) => self.wrap_unary(op),
            Err(e) => Query::Error(e),
        }
    }

    /// Y方向のへ値をリニアに減少させる。
    /// 指定した距離で0になる。
    pub fn falloff_linear_y<Z: Into<u8>, P>(self, z: Z, radius: u32, _policy: P) -> Self
    where
        P: MergePolicy<V> + Send + Sync,
        V: Mul<Output = V> + Div<Output = V> + Sub<Output = V> + TryFrom<u32> + Clone + Send + Sync,
        <V as TryFrom<u32>>::Error: Debug,
    {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        match FalloffLinearY::<P>::new(z, radius) {
            Ok(op) => self.wrap_unary(op),
            Err(e) => Query::Error(e),
        }
    }
}
