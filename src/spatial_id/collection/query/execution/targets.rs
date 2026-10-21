use crate::{RangeId, SpatialId};
use alloc::vec;
use alloc::vec::Vec;

/// [`Query::get`](crate::Query::get) に評価する範囲として渡せるもの。
pub trait IntoTargets {
    fn into_targets(self) -> Vec<RangeId>;
}

impl IntoTargets for core::ops::RangeFull {
    fn into_targets(self) -> Vec<RangeId> {
        vec![RangeId::WHOLE]
    }
}

impl<T: SpatialId> IntoTargets for T {
    fn into_targets(self) -> Vec<RangeId> {
        vec![self.into()]
    }
}

impl<T: SpatialId, const N: usize> IntoTargets for [T; N] {
    fn into_targets(self) -> Vec<RangeId> {
        self.into_iter().map(Into::into).collect()
    }
}

impl<T: SpatialId> IntoTargets for Vec<T> {
    fn into_targets(self) -> Vec<RangeId> {
        self.into_iter().map(Into::into).collect()
    }
}

impl<T: SpatialId> IntoTargets for &[T] {
    fn into_targets(self) -> Vec<RangeId> {
        self.iter().cloned().map(Into::into).collect()
    }
}
