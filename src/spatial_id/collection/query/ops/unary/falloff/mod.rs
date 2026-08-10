pub mod falloff_f;
pub mod falloff_x;
pub mod falloff_y;
pub mod primitive;
pub mod query;

#[cfg(test)]
mod test;

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

impl<V> crate::spatial_id::collection::query::grid::GridAttenuator<V> for Attenuator<V>
where
    V: Mul<Output = V> + Div<Output = V> + Sub<Output = V> + TryFrom<u32> + Clone,
    <V as TryFrom<u32>>::Error: Debug,
{
    #[inline(always)]
    fn attenuate(&self, value: &V, distance: u32) -> V {
        self.attenuate(value, distance)
    }
}
