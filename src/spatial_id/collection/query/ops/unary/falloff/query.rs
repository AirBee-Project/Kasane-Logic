use super::{FalloffPattern, falloff_f::FalloffF, falloff_x::FalloffX, falloff_y::FalloffY};
use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::query::{execution::Query, merge_policy::MergePolicy};
use crate::spatial_id::helpers::Side;
use core::convert::TryFrom;
use core::fmt::Debug;
use core::ops::{Div, Mul, Sub};

impl<V: SafeValue + 'static> Query<V> {
    /// F方向へ値を減少させる。
    /// 指定した距離で0になる。
    pub fn falloff_f<Z: Into<u8>, P>(
        self,
        z: Z,
        radius: u32,
        direction: Option<Side>,
        pattern: FalloffPattern,
        _policy: P,
    ) -> Self
    where
        P: MergePolicy<V> + Send + Sync,
        V: Mul<Output = V> + Div<Output = V> + Sub<Output = V> + TryFrom<u32> + Clone + Send + Sync,
        <V as TryFrom<u32>>::Error: Debug,
    {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        match FalloffF::<P>::new(z, radius, direction, pattern) {
            Ok(op) => self.wrap_unary(op),
            Err(e) => Query::Error(e),
        }
    }

    /// X方向へ値を減少させる。
    /// 指定した距離で0になる。
    pub fn falloff_x<Z: Into<u8>, P>(
        self,
        z: Z,
        radius: u32,
        direction: Option<Side>,
        pattern: FalloffPattern,
        _policy: P,
    ) -> Self
    where
        P: MergePolicy<V> + Send + Sync,
        V: Mul<Output = V> + Div<Output = V> + Sub<Output = V> + TryFrom<u32> + Clone + Send + Sync,
        <V as TryFrom<u32>>::Error: Debug,
    {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        match FalloffX::<P>::new(z, radius, direction, pattern) {
            Ok(op) => self.wrap_unary(op),
            Err(e) => Query::Error(e),
        }
    }

    /// Y方向へ値を減少させる。
    /// 指定した距離で0になる。
    pub fn falloff_y<Z: Into<u8>, P>(
        self,
        z: Z,
        radius: u32,
        direction: Option<Side>,
        pattern: FalloffPattern,
        _policy: P,
    ) -> Self
    where
        P: MergePolicy<V> + Send + Sync,
        V: Mul<Output = V> + Div<Output = V> + Sub<Output = V> + TryFrom<u32> + Clone + Send + Sync,
        <V as TryFrom<u32>>::Error: Debug,
    {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        match FalloffY::<P>::new(z, radius, direction, pattern) {
            Ok(op) => self.wrap_unary(op),
            Err(e) => Query::Error(e),
        }
    }
}
