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
            |id, value| id.falloff_y(z, radius, self.direction, self.pattern, value),
            |a: &V, b: &V| P::resolve(a.clone(), b.clone()),
        )?;
        *target = WorkingTree::from_core(rebuilt);
        Ok(())
    }

    fn inverse_bounds(&self, mut bounds: crate::RangeId) -> Option<crate::RangeId> {
        let target_z = bounds.z();
        let z = self.z.get();
        let max_z = z.max(target_z);
        let shift_z = max_z - z;
        let scale_t = max_z - target_z;

        let delta = self.radius * (1u32 << shift_z);
        let mut min_delta = delta;
        let mut max_delta = delta;
        if let Some(side) = self.direction {
            if side == crate::spatial_id::helpers::Side::Upper {
                min_delta = 0;
            } else {
                max_delta = 0;
            }
        }

        let y_min_max_z = bounds.y()[0] * (1u32 << scale_t);
        let y_max_max_z = (bounds.y()[1] + 1) * (1u32 << scale_t) - 1;

        let max_len = 1u32 << max_z;
        let new_min_max_z = y_min_max_z.saturating_sub(min_delta);
        let new_max_max_z = y_max_max_z.saturating_add(max_delta).min(max_len - 1);

        if new_min_max_z <= new_max_max_z {
            let new_min_target = new_min_max_z >> scale_t;
            let new_max_target = new_max_max_z >> scale_t;
            bounds.set_y([new_min_target, new_max_target]).unwrap();
            Some(bounds)
        } else {
            None
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
            "falloff_y(z={}, r={}, dir={}, pat={:?}, {})",
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
        Ok(grid.falloff::<P, _>(GridAxis::Y, self.z, self.radius, self.direction, &atten))
    }
}
