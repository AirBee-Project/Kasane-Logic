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
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FalloffAxis {
    F,
    X,
    Y,
}

pub struct FalloffLinearFxy<P> {
    z: ZoomLevel,
    f_radius: u32,
    x_radius: u32,
    y_radius: u32,
    order: Vec<FalloffAxis>,
    _marker: PhantomData<P>,
}

impl<P> FalloffLinearFxy<P> {}

impl<W, P> UnaryOperator<W> for FalloffLinearFxy<P>
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
        let f = (self.f_radius * 2 + 1) as f32;
        let x = (self.x_radius * 2 + 1) as f32;
        let y = (self.y_radius * 2 + 1) as f32;
        f * x * y
    }

    fn effective_expansion_ratio(&self, bbox: Option<&crate::RangeId>) -> f32 {
        if let Some(bb) = bbox {
            let x = bb.x();
            let y = bb.y();
            let f = bb.f();

            let s_x = x[1].saturating_sub(x[0]) as f32 + 1.0;
            let s_y = y[1].saturating_sub(y[0]) as f32 + 1.0;
            let s_f = f[1].saturating_sub(f[0]).max(0) as f32 + 1.0;

            let r_x = self.x_radius as f32;
            let r_y = self.y_radius as f32;
            let r_f = self.f_radius as f32;

            let e_x = (s_x + 2.0 * r_x) / s_x;
            let e_y = (s_y + 2.0 * r_y) / s_y;
            let e_f = (s_f + 2.0 * r_f) / s_f;

            e_x * e_y * e_f
        } else {
            <Self as UnaryOperator<W>>::expansion_ratio(self)
        }
    }

    fn try_merge(
        &self,
        _other: &dyn UnaryOperator<W>,
    ) -> Option<alloc::boxed::Box<dyn UnaryOperator<W>>> {
        // マージ（Fxy化）による次元の呪い（Cullingの遅延）を防ぐため、
        // 意図的にマージを無効化し、各軸ごとに逐次適用・合成させる。
        // MergeAccumulatorのTrait実装自体は構造的に残している。
        None
    }

    fn run(&self, target: &mut W) -> Result<(), Error> {
        if self.f_radius == 0 && self.x_radius == 0 && self.y_radius == 0 {
            return Ok(());
        }
        let z = self.z.get();
        let (f_radius, x_radius, y_radius) = (self.f_radius, self.x_radius, self.y_radius);

        // 反映先が非単射（直方体近傍が互いに重なる）なので merge_with で合成する。
        *target = target.map_rebuild_with(
            |id, value| id.falloff_linear_fxy(z, f_radius, x_radius, y_radius, &self.order, value),
            |a: &W::Value, b: &W::Value| P::resolve(a.clone(), b.clone()),
        )?;
        Ok(())
    }

    fn validate(&self) -> Result<(), crate::Error> {
        Ok(())
    }
}

fn merge_falloff_axis(cur: u32, add: u32) -> Option<u32> {
    if cur == 0 {
        Some(add)
    } else if add == 0 {
        Some(cur)
    } else {
        None // Conflict! Mathematical restriction on linear falloff convolution
    }
}

impl<W, P> crate::spatial_id::collection::query::traits::MergeAccumulator<W> for FalloffLinearFxy<P>
where
    W: WorkingTree + 'static,
    W::Value:
        Mul<Output = W::Value> + Div<Output = W::Value> + Sub<Output = W::Value> + TryFrom<u32>,
    <W::Value as TryFrom<u32>>::Error: Debug,
    P: MergePolicy<W::Value> + Send + Sync + 'static,
    W::Value: 'static,
{
    fn seed(op: &dyn UnaryOperator<W>) -> Option<Self> {
        let any = op.as_any();
        if let Some(o) = any.downcast_ref::<FalloffLinearFxy<P>>() {
            return Some(Self {
                z: o.z,
                f_radius: o.f_radius,
                x_radius: o.x_radius,
                y_radius: o.y_radius,
                order: o.order.clone(),
                _marker: core::marker::PhantomData,
            });
        }
        if let Some(o) = any.downcast_ref::<super::falloff_linear_x::FalloffLinearX<P>>() {
            return Some(Self {
                z: o.z,
                f_radius: 0,
                x_radius: o.radius,
                y_radius: 0,
                order: vec![FalloffAxis::X],
                _marker: core::marker::PhantomData,
            });
        }
        if let Some(o) = any.downcast_ref::<super::falloff_linear_y::FalloffLinearY<P>>() {
            return Some(Self {
                z: o.z,
                f_radius: 0,
                x_radius: 0,
                y_radius: o.radius,
                order: vec![FalloffAxis::Y],
                _marker: core::marker::PhantomData,
            });
        }
        if let Some(o) = any.downcast_ref::<super::falloff_linear_f::FalloffLinearF<P>>() {
            return Some(Self {
                z: o.z,
                f_radius: o.radius,
                x_radius: 0,
                y_radius: 0,
                order: vec![FalloffAxis::F],
                _marker: core::marker::PhantomData,
            });
        }
        None
    }

    fn absorb(&mut self, op: &dyn UnaryOperator<W>) -> bool {
        let Some(delta) =
            <Self as crate::spatial_id::collection::query::traits::MergeAccumulator<W>>::seed(op)
        else {
            return false;
        };
        if self.z != delta.z {
            return false;
        }
        let (Some(f), Some(x), Some(y)) = (
            merge_falloff_axis(self.f_radius, delta.f_radius),
            merge_falloff_axis(self.x_radius, delta.x_radius),
            merge_falloff_axis(self.y_radius, delta.y_radius),
        ) else {
            return false;
        };
        self.f_radius = f;
        self.x_radius = x;
        self.y_radius = y;
        self.order.extend(delta.order);
        true
    }
}
