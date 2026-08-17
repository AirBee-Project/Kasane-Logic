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

pub struct FalloffX<P> {
    pub z: ZoomLevel,
    pub radius: u32,
    pub direction: Option<Side>,
    pub pattern: FalloffPattern,
    _marker: PhantomData<P>,
}

impl<P> FalloffX<P> {
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

impl<V: SafeValue + 'static, P> UnaryOperator<V> for FalloffX<P>
where
    V: Mul<Output = V> + Div<Output = V> + Sub<Output = V> + TryFrom<u32>,
    <V as TryFrom<u32>>::Error: Debug,
    P: MergePolicy<V> + Send + Sync + 'static,
{
    fn commutativity_info(&self) -> CommutativityInfo {
        if !P::IS_COMMUTATIVE {
            return CommutativityInfo::None;
        }
        CommutativityInfo::Separable {
            policy: Some(core::any::TypeId::of::<P>()),
        }
    }
    fn as_any(&self) -> &dyn core::any::Any {
        self
    }

    fn expansion_ratio(&self) -> f64 {
        (self.radius * 2 + 1) as f64
    }

    fn run(&self, target: &mut WorkingTree<V>) -> Result<(), Error> {
        if self.radius == 0 {
            return Ok(());
        }
        let z = self.z.get();
        let radius = self.radius;

        // 反映先が非単射（近傍が互いに重なる）なので merge_with で合成する。
        let rebuilt = target.core().map_rebuild_with(
            |id, value| id.falloff_x(z, radius, self.direction, self.pattern, value),
            |a: &V, b: &V| P::resolve(a.clone(), b.clone()),
        )?;
        *target = WorkingTree::from_core(rebuilt);
        Ok(())
    }

    fn inverse_bounds(&self, bounds: crate::RangeId) -> alloc::vec::Vec<crate::RangeId> {
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

        let x_min_max_z = (bounds.x()[0] as i64) * (1i64 << scale_t);
        let x_max_max_z = ((bounds.x()[1] as i64) + 1) * (1i64 << scale_t) - 1;

        let new_min_max_z = x_min_max_z - min_delta;
        let new_max_max_z = x_max_max_z + max_delta;

        let max_len = 1i64 << max_z;
        let new_min_max_z_wrapped = new_min_max_z.rem_euclid(max_len);
        let new_max_max_z_wrapped = new_max_max_z.rem_euclid(max_len);

        let mut x_ranges = alloc::vec::Vec::new();
        if new_max_max_z - new_min_max_z >= max_len {
            x_ranges.push((0, max_len - 1));
        } else if new_min_max_z_wrapped <= new_max_max_z_wrapped {
            x_ranges.push((new_min_max_z_wrapped, new_max_max_z_wrapped));
        } else {
            x_ranges.push((new_min_max_z_wrapped, max_len - 1));
            x_ranges.push((0, new_max_max_z_wrapped));
        }

        let mut res = alloc::vec::Vec::new();
        for (min_max_z, max_max_z) in x_ranges {
            let mut new_bounds = bounds.clone();
            let new_min_target = (min_max_z >> scale_t) as u32;
            let new_max_target = (max_max_z >> scale_t) as u32;
            new_bounds.set_x([new_min_target, new_max_target]).unwrap();
            res.push(new_bounds);
        }
        res
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
            "falloff_x(z={}, r={}, dir={}, pat={:?}, {})",
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

    #[allow(private_interfaces)]
    fn apply_to_grid(
        &self,
        grid: &mut crate::spatial_id::collection::query::grid::UniformGrid<V>,
    ) -> Result<crate::spatial_id::collection::query::grid::Applied, crate::Error> {
        if !P::IS_COMMUTATIVE || self.radius == 0 {
            return Ok(crate::spatial_id::collection::query::grid::Applied::Unsupported);
        }
        let atten = super::Attenuator::new(self.radius, self.pattern);
        Ok(grid.falloff::<P, _>(GridAxis::X, self.z, self.radius, self.direction, &atten))
    }
}
