pub mod primitive;
pub mod query;

#[cfg(test)]
mod test;

use crate::spatial_id::collection::flex_tree::core::NoSummary;
use crate::spatial_id::collection::query::{
    merge_policy::MergePolicy,
    send_sync::SafeValue,
    traits::{QueryIter, UnaryOperator},
};
use crate::spatial_id::helpers::Side;
use crate::{Error, RangeId, SpatialIdTable, ZoomLevel};
use alloc::vec::Vec;
use core::convert::TryFrom;
use core::fmt::Debug;
use core::marker::PhantomData;
use core::ops::{Div, Mul, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FalloffAxis {
    F,
    X,
    Y,
}

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

pub struct Falloff<P> {
    pub axis: FalloffAxis,
    pub z: ZoomLevel,
    pub radius: u32,
    pub direction: Option<Side>,
    pub pattern: FalloffPattern,
    _marker: PhantomData<fn() -> P>,
}

impl<P> Falloff<P> {
    pub fn new<T: Into<u8>>(
        axis: FalloffAxis,
        z: T,
        radius: u32,
        direction: Option<Side>,
        pattern: FalloffPattern,
    ) -> Result<Self, Error> {
        let z = ZoomLevel::new(z.into())?;
        Ok(Self {
            axis,
            z,
            radius,
            direction,
            pattern,
            _marker: PhantomData,
        })
    }
}

impl<V: SafeValue + 'static, P> UnaryOperator<V> for Falloff<P>
where
    V: Mul<Output = V> + Div<Output = V> + Sub<Output = V> + TryFrom<u32>,
    <V as TryFrom<u32>>::Error: Debug,
    P: MergePolicy<V>,
{
    fn run<'a>(&'a self, input: QueryIter<'a, V>) -> Result<QueryIter<'a, V>, Error> {
        if self.radius == 0 {
            return Ok(input);
        }
        let z = self.z.get();
        let radius = self.radius;

        // 反映先が非単射（近傍が互いに重なる）なので、重なりは MergePolicy で合成する。
        let resolve = |a: &V, b: &V| P::resolve(a.clone(), b.clone());
        let mut rebuilt = SpatialIdTable::<V, NoSummary>::default();
        for item in input {
            let (id, value) = item?;
            for (spread, spread_value) in
                id.falloff(self.axis, z, radius, self.direction, self.pattern, &value)?
            {
                rebuilt.inner.insert_with(spread, spread_value, resolve);
            }
        }
        Ok(rebuilt.into())
    }

    fn reverse_target(&self, bounds: RangeId, out: &mut Vec<RangeId>) {
        let r = self.radius as i64;
        // 順方向が Upper (+方向) へ広がるなら、出力へ届きうる入力は Lower (-方向) にある
        let d = match self.direction {
            Some(Side::Upper) => [-r, 0],
            Some(Side::Lower) => [0, r],
            None => [-r, r],
        };
        let (f, x, y) = match self.axis {
            FalloffAxis::F => (d, [0, 0], [0, 0]),
            FalloffAxis::X => ([0, 0], d, [0, 0]),
            FalloffAxis::Y => ([0, 0], [0, 0], d),
        };
        bounds.shift_range(self.z.get(), f, x, y, out);
    }

    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}
