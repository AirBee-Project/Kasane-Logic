pub mod primitive;
pub mod query;

#[cfg(test)]
mod test;

use crate::spatial_id::collection::flex_tree::core::{FlexTreeCore, NoSummary};
use crate::spatial_id::collection::query::{
    merge_policy::MergePolicy,
    ops::unary::sort_by_flex_id,
    send_sync::SafeValue,
    traits::{QueryIter, UnaryOperator},
};
use crate::spatial_id::helpers::Side;
use crate::{Error, RangeId, SpatialIdTable, ZoomLevel};
use alloc::vec;
use alloc::vec::Vec;
use core::convert::TryFrom;
use core::marker::PhantomData;
use core::ops::{Add, Div, Mul, Rem, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FalloffAxis {
    F,
    X,
    Y,
}

impl FalloffAxis {
    /// `radius` がズーム `z` でのこの軸の範囲に収まるか検証する。
    pub(crate) fn check_radius(self, z: ZoomLevel, radius: u32) -> Result<(), Error> {
        let max = match self {
            Self::F => z.f_max() as u32,
            Self::X | Self::Y => z.xy_max(),
        };
        if radius > max {
            return Err(Error::InvalidQueryParameter(
                "falloff の半径が軸の範囲を超えている",
            ));
        }
        Ok(())
    }
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
    V: Add<Output = V>
        + Sub<Output = V>
        + Mul<Output = V>
        + Div<Output = V>
        + Rem<Output = V>
        + TryFrom<u64>
        + Clone,
{
    pub fn new(radius: u32, pattern: FalloffPattern) -> Result<Self, Error> {
        let too_large = || Error::InvalidQueryParameter("falloff の半径が値の型に収まらない");
        let to_v = |n: u64| V::try_from(n).map_err(|_| too_large());
        // `V` の乗算で確かめるとあふれたときに panic するので、u64 で2乗してから変換して確かめる
        let sq = |n: u64| n.checked_mul(n).ok_or_else(too_large);
        let radius = u64::from(radius);
        // `attenuate` の途中の積は分母の2乗を超えないので、それが `V` に収まれば途中であふれない
        Ok(match pattern {
            FalloffPattern::Linear => {
                to_v(sq(radius)?)?;
                Self::Linear {
                    radius: to_v(radius)?,
                }
            }
            FalloffPattern::QuadraticIn => {
                to_v(sq(sq(radius)?)?)?;
                Self::QuadraticIn {
                    r_sq: to_v(sq(radius)?)?,
                }
            }
            FalloffPattern::QuadraticOut => {
                to_v(sq(sq(radius)?)?)?;
                Self::QuadraticOut {
                    radius: to_v(radius)?,
                    r_sq: to_v(sq(radius)?)?,
                }
            }
        })
    }

    /// `value * num / den` を、途中の積が `value` と `den` の2乗を超えないように求める。
    #[inline(always)]
    pub fn attenuate(&self, value: &V, distance: u32) -> V {
        let d = V::try_from(u64::from(distance)).unwrap_or_else(|_| unreachable!("距離は半径以下"));
        let (num, den) = match self {
            Self::Linear { radius } => (radius.clone() - d, radius),
            Self::QuadraticIn { r_sq } => (r_sq.clone() - d.clone() * d, r_sq),
            Self::QuadraticOut { radius, r_sq } => {
                let r_minus_d = radius.clone() - d;
                (r_minus_d.clone() * r_minus_d, r_sq)
            }
        };
        // value = q * den + rem と分けると、num <= den なので q * num は value を、rem * num は den の2乗を超えない。
        // 整数の切り捨ても value * num / den と一致する
        let rem = value.clone() % den.clone();
        (value.clone() - rem.clone()) / den.clone() * num.clone() + rem * num / den.clone()
    }
}

/// 値を軸方向へ広げながら減衰させる演算子。
///
/// 半径は、線形の減衰では半径の2乗、2次の減衰では半径の4乗が `V` と u64 に収まる範囲で指定する。
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
    V: Add<Output = V>
        + Sub<Output = V>
        + Mul<Output = V>
        + Div<Output = V>
        + Rem<Output = V>
        + TryFrom<u64>
        + Clone,
    P: MergePolicy<V>,
{
    fn run<'a>(&'a self, input: QueryIter<'a, V>) -> Result<QueryIter<'a, V>, Error> {
        if self.radius == 0 {
            return Ok(input);
        }
        let z = self.z.get();

        let mut spread = Vec::new();
        for item in input {
            let (id, value) = item?;
            spread.extend(id.falloff(
                self.axis,
                z,
                self.radius,
                self.direction,
                self.pattern,
                &value,
            )?);
        }
        sort_by_flex_id(&mut spread);

        // 近傍は互いに重なる。2件ずつ resolve すると Average のように resolve_many を持つ規則で値がずれるので、
        // 重なった値をすべて集めてから resolve_many で1回だけ合成する
        let mut gathered = FlexTreeCore::<Vec<V>, NoSummary>::default();
        let mut spread = spread.into_iter().peekable();
        while let Some((id, value)) = spread.next() {
            let mut values = vec![value];
            while let Some((_, value)) = spread.next_if(|(next, _)| *next == id) {
                values.push(value);
            }
            // ズームの違う寄与は ID が一致しないまま重なるので、木の上で値の列をつなぐ
            gathered.insert_with([id], values, |a, b| [a.as_slice(), b].concat());
        }

        let mut rebuilt = SpatialIdTable::<V, NoSummary>::default();
        for (id, values) in gathered.iter() {
            if let Some(value) = P::resolve_many(values.iter().cloned()) {
                rebuilt.inner.insert([id], value);
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
        self.axis.check_radius(self.z, self.radius)?;
        Attenuator::<V>::new(self.radius, self.pattern).map(drop)
    }
}
