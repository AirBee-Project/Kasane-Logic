use crate::SpatialIdTable;
use crate::spatial_id::collection::flex_tree::core::NoSummary;
use alloc::boxed::Box;

use crate::spatial_id::collection::query::send_sync::SafeValue;
use core::convert::TryFrom;
use core::fmt::Debug;
use core::marker::PhantomData;
use core::ops::{Div, Mul, Sub};

use crate::{
    Error, ZoomLevel,
    spatial_id::collection::query::{
        merge_policy::MergePolicy,
        traits::{QueryIter, UnaryOperator},
    },
};

use super::FalloffPattern;
use crate::spatial_id::helpers::Side;

pub struct FalloffY<P> {
    pub z: ZoomLevel,
    pub radius: u32,
    pub direction: Option<Side>,
    pub pattern: FalloffPattern,
    _marker: PhantomData<P>,
}

impl<P> FalloffY<P> {
    pub fn new<T: Into<u8>>(
        z: T,
        radius: u32,
        direction: Option<Side>,
        pattern: FalloffPattern,
    ) -> Result<Self, Error> {
        let z = ZoomLevel::new(z.into())?;
        Ok(Self {
            z,
            radius,
            direction,
            pattern,
            _marker: PhantomData,
        })
    }
}

impl<V: SafeValue + 'static, P> UnaryOperator<V> for FalloffY<P>
where
    V: Mul<Output = V> + Div<Output = V> + Sub<Output = V> + TryFrom<u32>,
    <V as TryFrom<u32>>::Error: Debug,
    P: MergePolicy<V> + Send + Sync + 'static,
{
    fn run<'a>(&self, input: QueryIter<'a, V>) -> Result<QueryIter<'a, V>, Error> {
        if self.radius == 0 {
            return Ok(input);
        }
        let target: SpatialIdTable<V, NoSummary> = input.collect::<Result<_, _>>()?;
        let z = self.z.get();
        let radius = self.radius;

        // 反映先が非単射（近傍が互いに重なる）なので、重なりは MergePolicy で合成する。
        let resolve = |a: &V, b: &V| P::resolve(a.clone(), b.clone());
        let mut rebuilt = SpatialIdTable::<V, NoSummary>::default();
        for (id, value) in target.iter() {
            for (spread, spread_value) in
                id.falloff_y(z, radius, self.direction, self.pattern, value)?
            {
                rebuilt.inner.insert_with(spread, spread_value, resolve);
            }
        }
        Ok(Box::new(rebuilt.into_iter().map(Ok)))
    }

    fn reverse_target(&self, bounds: crate::RangeId) -> Option<crate::RangeId> {
        let z = self.z.get();
        let target_z = z.max(bounds.z());

        let delta = (self.radius as i64) * (1i64 << (target_z - z));
        let mut min_delta = delta;
        let mut max_delta = delta;
        if let Some(side) = self.direction {
            if side == crate::spatial_id::helpers::Side::Upper {
                min_delta = 0;
            } else {
                max_delta = 0;
            }
        }

        bounds
            .y_edges_shift(target_z, -min_delta, max_delta)
            .unwrap()
    }

    fn validate(&self) -> Result<(), crate::Error> {
        Ok(())
    }
}
