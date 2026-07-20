use alloc::boxed::Box;
use core::convert::TryFrom;
use core::fmt::Debug;
use core::marker::PhantomData;
use core::ops::{Div, Mul, Sub};

use crate::{
    Error, ZoomLevel,
    spatial_id::collection::query::{
        merge_policy::MergePolicy,
        traits::{UnaryOperator, WorkingTree},
    },
};

pub struct FalloffLinearFxy<P> {
    z: ZoomLevel,
    f_radius: u32,
    x_radius: u32,
    y_radius: u32,
    _marker: PhantomData<P>,
}

impl<P> FalloffLinearFxy<P> {
    pub fn new<T: Into<u8>>(
        z: T,
        f_radius: u32,
        x_radius: u32,
        y_radius: u32,
    ) -> Result<Self, Error> {
        let z = ZoomLevel::new(z.into())?;
        Ok(Self {
            z,
            f_radius,
            x_radius,
            y_radius,
            _marker: PhantomData,
        })
    }
}

impl<W, P> UnaryOperator<W> for FalloffLinearFxy<P>
where
    W: WorkingTree,
    W::Value:
        Mul<Output = W::Value> + Div<Output = W::Value> + Sub<Output = W::Value> + TryFrom<u32>,
    <W::Value as TryFrom<u32>>::Error: Debug,
    P: MergePolicy<W::Value> + Send + Sync,
{
    fn run(&self, target: &mut W) -> Result<(), Box<dyn core::error::Error + 'static>> {
        if self.f_radius == 0 && self.x_radius == 0 && self.y_radius == 0 {
            return Ok(());
        }
        let z = self.z.get();
        let (f_radius, x_radius, y_radius) = (self.f_radius, self.x_radius, self.y_radius);

        // 反映先が非単射（直方体近傍が互いに重なる）なので merge_with で合成する。
        *target = target.map_rebuild_with(
            |id, value| id.falloff_linear_fxy(z, f_radius, x_radius, y_radius, value),
            |a: &W::Value, b: &W::Value| P::resolve(a.clone(), b.clone()),
        )?;
        Ok(())
    }
}
