use crate::spatial_id::collection::query::execution::group_commutative::types::CommutativityInfo;
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

pub struct FalloffLinearY<P> {
    pub z: ZoomLevel,
    pub radius: u32,
    _marker: PhantomData<P>,
}

impl<P> FalloffLinearY<P> {
    pub fn new<T: Into<u8>>(z: T, radius: u32) -> Result<Self, Error> {
        let z = ZoomLevel::new(z.into())?;
        Ok(Self {
            z,
            radius,
            _marker: PhantomData,
        })
    }
}

impl<W, P> UnaryOperator<W> for FalloffLinearY<P>
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

    fn effective_expansion_ratio(&self, bbox: Option<&crate::RangeId>) -> f32 {
        if let Some(bb) = bbox {
            let y = bb.y();
            let s = y[1].saturating_sub(y[0]) as f32 + 1.0;
            let r = self.radius as f32;
            (s + 2.0 * r) / s
        } else {
            <Self as UnaryOperator<W>>::expansion_ratio(self)
        }
    }

    fn run(&self, target: &mut W) -> Result<(), Error> {
        if self.radius == 0 {
            return Ok(());
        }
        let z = self.z.get();
        let radius = self.radius;

        // 反映先が非単射（近傍が互いに重なる）なので merge_with で合成する。
        *target = target.map_rebuild_with(
            |id, value| id.falloff_linear_y(z, radius, value),
            |a: &W::Value, b: &W::Value| P::resolve(a.clone(), b.clone()),
        )?;
        Ok(())
    }

    fn validate(&self) -> Result<(), crate::Error> {
        Ok(())
    }

    fn try_merge(&self, _other: &dyn UnaryOperator<W>) -> Option<Box<dyn UnaryOperator<W>>> {
        // マージ（Fxy化）による次元の呪い（Cullingの遅延および3D直方体計算量の爆増）を防ぐため、
        // 意図的にマージを無効化し、各軸ごとに1D分離適用・合成させる。
        None
    }
}
