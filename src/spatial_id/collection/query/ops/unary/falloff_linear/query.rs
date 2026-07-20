use super::{
    falloff_linear_f::FalloffLinearF, falloff_linear_fxy::FalloffLinearFxy,
    falloff_linear_x::FalloffLinearX, falloff_linear_y::FalloffLinearY,
};
use crate::{
    SpatialIdCollection,
    spatial_id::collection::query::{execution::Query, merge_policy::MergePolicy},
};
use core::convert::TryFrom;
use core::fmt::Debug;
use core::ops::{Div, Mul, Sub};

impl<S: SpatialIdCollection> Query<S>
where
    S::Value: 'static,
{
    /// 3次元統合の FalloffLinear 演算を適用する
    pub fn falloff_linear_fxy<T: Into<u8>, P>(
        self,
        z: T,
        f_radius: u32,
        x_radius: u32,
        y_radius: u32,
        _policy: P,
    ) -> Self
    where
        P: MergePolicy<S::Value> + Send + Sync,
        S::Value: Mul<Output = S::Value>
            + Div<Output = S::Value>
            + Sub<Output = S::Value>
            + TryFrom<u32>
            + Clone
            + Send
            + Sync,
        <S::Value as TryFrom<u32>>::Error: Debug,
    {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        match FalloffLinearFxy::<P>::new(z, f_radius, x_radius, y_radius) {
            Ok(op) => self.wrap_unary(op),
            Err(e) => Query::Error(e),
        }
    }

    /// F方向の FalloffLinear 演算を適用する
    pub fn falloff_linear_f<Z: Into<u8>, P>(self, z: Z, radius: u32, _policy: P) -> Self
    where
        P: MergePolicy<S::Value> + Send + Sync,
        S::Value: Mul<Output = S::Value>
            + Div<Output = S::Value>
            + Sub<Output = S::Value>
            + TryFrom<u32>
            + Clone
            + Send
            + Sync,
        <S::Value as TryFrom<u32>>::Error: Debug,
    {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        match FalloffLinearF::<P>::new(z, radius) {
            Ok(op) => self.wrap_unary(op),
            Err(e) => Query::Error(e),
        }
    }

    /// X方向の FalloffLinear 演算を適用する
    pub fn falloff_linear_x<Z: Into<u8>, P>(self, z: Z, radius: u32, _policy: P) -> Self
    where
        P: MergePolicy<S::Value> + Send + Sync,
        S::Value: Mul<Output = S::Value>
            + Div<Output = S::Value>
            + Sub<Output = S::Value>
            + TryFrom<u32>
            + Clone
            + Send
            + Sync,
        <S::Value as TryFrom<u32>>::Error: Debug,
    {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        match FalloffLinearX::<P>::new(z, radius) {
            Ok(op) => self.wrap_unary(op),
            Err(e) => Query::Error(e),
        }
    }

    /// Y方向の FalloffLinear 演算を適用する
    pub fn falloff_linear_y<Z: Into<u8>, P>(self, z: Z, radius: u32, _policy: P) -> Self
    where
        P: MergePolicy<S::Value> + Send + Sync,
        S::Value: Mul<Output = S::Value>
            + Div<Output = S::Value>
            + Sub<Output = S::Value>
            + TryFrom<u32>
            + Clone
            + Send
            + Sync,
        <S::Value as TryFrom<u32>>::Error: Debug,
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
