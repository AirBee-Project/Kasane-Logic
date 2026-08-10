pub mod falloff_f;
pub mod falloff_x;
pub mod falloff_y;
pub mod primitive;
pub mod query;

#[cfg(test)]
mod test;

use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::query::grid::{GridAxis, GridOp};
use crate::spatial_id::collection::query::merge_policy::MergePolicy;
use crate::spatial_id::helpers::Side;
use crate::spatial_id::zoom_level::ZoomLevel;
use alloc::boxed::Box;
use core::convert::TryFrom;
use core::fmt::Debug;
use core::ops::{Div, Mul, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FalloffPattern {
    Linear,
    QuadraticIn,
    QuadraticOut,
}

pub(crate) enum Attenuator<V> {
    Linear { radius: V },
    QuadraticIn { r_sq: V },
    QuadraticOut { radius: V, r_sq: V },
}

impl<V> Attenuator<V>
where
    V: Mul<Output = V> + Div<Output = V> + Sub<Output = V> + TryFrom<u32> + Clone,
    <V as TryFrom<u32>>::Error: Debug,
{
    pub fn new(radius: u32, pattern: FalloffPattern) -> Self {
        let v_radius = V::try_from(radius).unwrap();
        match pattern {
            FalloffPattern::Linear => Self::Linear { radius: v_radius },
            FalloffPattern::QuadraticIn => {
                let r_sq = v_radius.clone() * v_radius.clone();
                Self::QuadraticIn { r_sq }
            }
            FalloffPattern::QuadraticOut => {
                let r_sq = v_radius.clone() * v_radius.clone();
                Self::QuadraticOut {
                    radius: v_radius,
                    r_sq,
                }
            }
        }
    }

    #[inline(always)]
    pub fn attenuate(&self, value: &V, distance: u32) -> V {
        let v_distance = V::try_from(distance).unwrap();
        match self {
            Self::Linear { radius } => {
                (value.clone() * (radius.clone() - v_distance)) / radius.clone()
            }
            Self::QuadraticIn { r_sq } => {
                let d_sq = v_distance.clone() * v_distance;
                (value.clone() * (r_sq.clone() - d_sq)) / r_sq.clone()
            }
            Self::QuadraticOut { radius, r_sq } => {
                let r_minus_d = radius.clone() - v_distance;
                let diff_sq = r_minus_d.clone() * r_minus_d;
                (value.clone() * diff_sq) / r_sq.clone()
            }
        }
    }
}

pub(crate) fn grid_op<V, P>(
    axis: GridAxis,
    z: ZoomLevel,
    radius: u32,
    direction: Option<Side>,
    pattern: FalloffPattern,
) -> Option<GridOp<V>>
where
    V: SafeValue + Mul<Output = V> + Div<Output = V> + Sub<Output = V> + TryFrom<u32> + 'static,
    <V as TryFrom<u32>>::Error: Debug,
    P: MergePolicy<V> + Send + Sync + 'static,
{
    if !P::IS_COMMUTATIVE {
        return None;
    }

    let atten: Box<crate::spatial_id::collection::query::grid::AttenFn<'static, V>> = match pattern
    {
        FalloffPattern::Linear => {
            let v_radius = V::try_from(radius).unwrap();
            Box::new(move |value: &V, distance: u32| {
                let v_distance = V::try_from(distance).unwrap();
                (value.clone() * (v_radius.clone() - v_distance)) / v_radius.clone()
            })
        }
        FalloffPattern::QuadraticIn => {
            let v_radius = V::try_from(radius).unwrap();
            let r_sq = v_radius.clone() * v_radius.clone();
            Box::new(move |value: &V, distance: u32| {
                let v_distance = V::try_from(distance).unwrap();
                let d_sq = v_distance.clone() * v_distance;
                (value.clone() * (r_sq.clone() - d_sq)) / r_sq.clone()
            })
        }
        FalloffPattern::QuadraticOut => {
            let v_radius = V::try_from(radius).unwrap();
            let r_sq = v_radius.clone() * v_radius.clone();
            Box::new(move |value: &V, distance: u32| {
                let v_distance = V::try_from(distance).unwrap();
                let r_minus_d = v_radius.clone() - v_distance;
                let diff_sq = r_minus_d.clone() * r_minus_d;
                (value.clone() * diff_sq) / r_sq.clone()
            })
        }
    };

    Some(GridOp::falloff(
        axis,
        z,
        radius,
        direction,
        atten,
        Box::new(|a, b| P::resolve(a.clone(), b.clone())),
    ))
}
