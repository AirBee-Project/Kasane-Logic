use alloc::boxed::Box;
use core::convert::TryFrom;
use core::fmt::Debug;
use core::marker::PhantomData;
use core::ops::{Div, Mul, Sub};

use crate::{
    Error, FlexTreeCore, ZoomLevel,
    spatial_id::collection::flex_tree::core::SafeValue,
    spatial_id::collection::query::{merge_policy::MergePolicy, traits::UnaryOperator},
};

pub struct FalloffLinearF<P> {
    z: ZoomLevel,
    radius: u32,
    _marker: PhantomData<P>,
}

impl<P> FalloffLinearF<P> {
    pub fn new<T: Into<u8>>(z: T, radius: u32) -> Result<Self, Error> {
        let z = ZoomLevel::new(z.into())?;
        Ok(Self {
            z,
            radius,
            _marker: PhantomData,
        })
    }
}

impl<V, P> UnaryOperator<V> for FalloffLinearF<P>
where
    V: SafeValue + Mul<Output = V> + Div<Output = V> + Sub<Output = V> + TryFrom<u32>,
    <V as TryFrom<u32>>::Error: Debug,
    P: MergePolicy<V> + Send + Sync,
{
    fn run(
        &self,
        target: &mut FlexTreeCore<V>,
    ) -> Result<(), Box<dyn core::error::Error + 'static>> {
        if self.radius == 0 {
            return Ok(());
        }
        let z = self.z.get();
        let radius = self.radius;

        // 反映先が非単射（近傍が互いに重なる）なので merge_with で合成する。
        *target = target.map_rebuild_with(
            |id, value| id.falloff_linear_f(z, radius, value),
            |a: &V, b: &V| P::resolve(a.clone(), b.clone()),
        )?;
        Ok(())
    }
}
