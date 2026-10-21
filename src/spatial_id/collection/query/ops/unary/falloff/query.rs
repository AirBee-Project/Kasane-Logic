use super::{Falloff, FalloffAxis, FalloffPattern};
use crate::spatial_id::collection::query::send_sync::SafeValue;
use crate::spatial_id::collection::query::{Query, merge_policy::MergePolicy};
use crate::spatial_id::helpers::Side;
use core::convert::TryFrom;
use core::ops::{Add, Div, Mul, Rem, Sub};

impl<V: SafeValue + 'static> Query<V> {
    fn falloff<Z: Into<u8>, P>(
        self,
        axis: FalloffAxis,
        z: Z,
        radius: u32,
        direction: Option<Side>,
        pattern: FalloffPattern,
    ) -> Self
    where
        P: MergePolicy<V> + Send + Sync,
        V: Add<Output = V>
            + Sub<Output = V>
            + Mul<Output = V>
            + Div<Output = V>
            + Rem<Output = V>
            + TryFrom<u64>
            + Clone
            + Send
            + Sync,
    {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        match Falloff::<P>::new(axis, z, radius, direction, pattern) {
            Ok(op) => self.wrap_unary(op),
            Err(e) => Query::Error(e),
        }
    }

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
        V: Add<Output = V>
            + Sub<Output = V>
            + Mul<Output = V>
            + Div<Output = V>
            + Rem<Output = V>
            + TryFrom<u64>
            + Clone
            + Send
            + Sync,
    {
        self.falloff::<Z, P>(FalloffAxis::F, z, radius, direction, pattern)
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
        V: Add<Output = V>
            + Sub<Output = V>
            + Mul<Output = V>
            + Div<Output = V>
            + Rem<Output = V>
            + TryFrom<u64>
            + Clone
            + Send
            + Sync,
    {
        self.falloff::<Z, P>(FalloffAxis::X, z, radius, direction, pattern)
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
        V: Add<Output = V>
            + Sub<Output = V>
            + Mul<Output = V>
            + Div<Output = V>
            + Rem<Output = V>
            + TryFrom<u64>
            + Clone
            + Send
            + Sync,
    {
        self.falloff::<Z, P>(FalloffAxis::Y, z, radius, direction, pattern)
    }
}
