use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::query::execution::group_commutative::types::CommutativityInfo;
use crate::spatial_id::collection::query::grid::GridAxis;
use crate::spatial_id::collection::query::working::WorkingTree;
use core::convert::TryFrom;
use core::fmt::Debug;
use core::marker::PhantomData;
use core::ops::{Div, Mul, Sub};

use crate::{
    Error, ZoomLevel,
    spatial_id::collection::query::{merge_policy::MergePolicy, traits::UnaryOperator},
};

use super::FalloffPattern;
use crate::spatial_id::helpers::Side;

pub struct FalloffF<P> {
    pub z: ZoomLevel,
    pub radius: u32,
    pub direction: Option<Side>,
    pub pattern: FalloffPattern,
    _marker: PhantomData<P>,
}

impl<P> FalloffF<P> {
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

impl<V: SafeValue + 'static, P> UnaryOperator<V> for FalloffF<P>
where
    V: Mul<Output = V> + Div<Output = V> + Sub<Output = V> + TryFrom<u32>,
    <V as TryFrom<u32>>::Error: Debug,
    P: MergePolicy<V> + Send + Sync + 'static,
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

    fn run(&self, target: &mut WorkingTree<V>) -> Result<(), Error> {
        if self.radius == 0 {
            return Ok(());
        }
        let z = self.z.get();
        let radius = self.radius;

        // 反映先が非単射（近傍が互いに重なる）なので merge_with で合成する。
        let rebuilt = target.core().map_rebuild_with(
            |id, value| id.falloff_f(z, radius, self.direction, self.pattern, value),
            |a: &V, b: &V| P::resolve(a.clone(), b.clone()),
        )?;
        *target = WorkingTree::from_core(rebuilt);
        Ok(())
    }

    fn inverse_bounds(&self, mut bounds: crate::RangeId) -> alloc::vec::Vec<crate::RangeId> {
        let target_z = bounds.z();
        let z = self.z.get();
        let max_z = z.max(target_z);
        let shift_z = max_z - z;
        let scale_t = max_z - target_z;

        let delta = (self.radius as i64) * (1i64 << shift_z);
        let mut min_delta = delta;
        let mut max_delta = delta;
        if let Some(side) = self.direction {
            if side == crate::spatial_id::helpers::Side::Upper {
                min_delta = 0;
            } else {
                max_delta = 0;
            }
        }

        let f_min_max_z = (bounds.f()[0] as i64) * (1i64 << scale_t);
        let f_max_max_z = ((bounds.f()[1] as i64) + 1) * (1i64 << scale_t) - 1;

        let max_z_obj = ZoomLevel::new(max_z).unwrap();
        let min_f = max_z_obj.f_min() as i64;
        let max_f = max_z_obj.f_max() as i64;

        let new_min_max_z = (f_min_max_z - min_delta).clamp(min_f, max_f);
        let new_max_max_z = (f_max_max_z + max_delta).clamp(min_f, max_f);

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
        let dir_str = match self.direction {
            None => "Both",
            Some(crate::spatial_id::helpers::Side::Upper) => "Upper",
            Some(crate::spatial_id::helpers::Side::Lower) => "Lower",
        };
        write!(
            f,
            "falloff_f(z={}, r={}, dir={}, pat={:?}, {})",
            self.z.get(),
            self.radius,
            dir_str,
            self.pattern,
            P::NAME
        )
    }

    fn grid_zoom(&self) -> Option<crate::ZoomLevel> {
        if !P::IS_COMMUTATIVE {
            return None;
        }
        Some(self.z)
    }

    fn apply_to_grid(
        &self,
        grid: &mut crate::spatial_id::collection::query::grid::UniformGrid<V>,
    ) -> Result<crate::spatial_id::collection::query::grid::Applied, crate::Error> {
        if !P::IS_COMMUTATIVE || self.radius == 0 {
            return Ok(crate::spatial_id::collection::query::grid::Applied::Unsupported);
        }
        let atten = super::Attenuator::new(self.radius, self.pattern);
        Ok(grid.falloff::<P, _>(GridAxis::F, self.z, self.radius, self.direction, &atten))
    }
}
