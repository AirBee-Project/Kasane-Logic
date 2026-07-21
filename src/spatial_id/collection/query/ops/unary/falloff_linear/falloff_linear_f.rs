use crate::spatial_id::collection::query::execution::group_commutative::types::CommutativityInfo;
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

pub struct FalloffLinearF<P> {
    pub z: ZoomLevel,
    pub radius: u32,
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

impl<W, P> UnaryOperator<W> for FalloffLinearF<P>
where
    W: WorkingTree + 'static,
    W::Value:
        Mul<Output = W::Value> + Div<Output = W::Value> + Sub<Output = W::Value> + TryFrom<u32>,
    <W::Value as TryFrom<u32>>::Error: Debug,
    P: MergePolicy<W::Value> + Send + Sync + 'static,
    W::Value: 'static,
{
    fn commutativity_info(&self) -> CommutativityInfo {
        CommutativityInfo::separable_with_policy::<P>(P::IS_COMMUTATIVE)
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }

    fn expansion_ratio(&self) -> f32 {
        (self.radius * 2 + 1) as f32
    }

    fn run(&self, target: &mut W) -> Result<(), Error> {
        if self.radius == 0 {
            return Ok(());
        }
        let z = self.z.get();
        let radius = self.radius;

        // 反映先が非単射（近傍が互いに重なる）なので merge_with で合成する。
        *target = target.map_rebuild_with(
            |id, value| id.falloff_linear_f(z, radius, value),
            |a: &W::Value, b: &W::Value| P::resolve(a.clone(), b.clone()),
        )?;
        Ok(())
    }

    fn inverse_bounds(&self, mut bounds: crate::RangeId) -> alloc::vec::Vec<crate::RangeId> {
        let target_z = bounds.z();
        let z = self.z.get();
        let max_z = z.max(target_z);
        let shift_z = max_z - z;
        let scale_t = max_z - target_z;
        
        let delta = (self.radius as i64) * (1i64 << shift_z);
        
        let f_min_max_z = (bounds.f()[0] as i64) * (1i64 << scale_t);
        let f_max_max_z = ((bounds.f()[1] as i64) + 1) * (1i64 << scale_t) - 1;
        
        let max_z_obj = ZoomLevel::new(max_z).unwrap();
        let min_f = max_z_obj.f_min() as i64;
        let max_f = max_z_obj.f_max() as i64;
        
        let new_min_max_z = (f_min_max_z - delta).clamp(min_f, max_f);
        let new_max_max_z = (f_max_max_z + delta).clamp(min_f, max_f);
        
        if new_min_max_z <= new_max_max_z {
            let new_min_target = (new_min_max_z >> scale_t) as i32;
            let new_max_target = (new_max_max_z >> scale_t) as i32;
            bounds.set_f([new_min_target, new_max_target]).unwrap();
            alloc::vec![bounds]
        } else {
            alloc::vec![]
        }
    }

    fn validate(&self) -> Result<(), crate::Error> {
        Ok(())
    }

    fn fmt_op(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "falloff_linear_f(z={}, r={}, {})",
            self.z.get(),
            self.radius,
            P::NAME
        )
    }
}
