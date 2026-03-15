#[cfg(any(test, feature = "random"))]
use rand::Rng;
#[cfg(any(test, feature = "random"))]
use std::ops::RangeInclusive;

#[cfg(any(test))]
use proptest::prelude::*;

#[cfg(any(test, feature = "random"))]
use crate::MAX_ZOOM_LEVEL;

use crate::SingleId;
#[cfg(any(test, feature = "random"))]
use crate::{F_MAX, F_MIN, XY_MAX};

impl SingleId {
    ///ランダムな[SingleId]を作成する
    #[cfg(any(test, feature = "random"))]
    pub fn random() -> Self {
        Self::random_within(0..=MAX_ZOOM_LEVEL as u8)
    }

    ///特定のズームレベルにおいて、ランダムな[SingleId]を作成する
    #[cfg(any(test, feature = "random"))]
    pub fn random_at(z: u8) -> Self {
        Self::random_within(z..=z)
    }

    ///特定のズームレベル間において、ランダムな[SingleId]を作成する
    #[cfg(any(test, feature = "random"))]
    pub fn random_within(z: RangeInclusive<u8>) -> Self {
        use rand::Rng;

        let mut rng = rand::rng();
        let start = *z.start();
        let end = (*z.end()).min(MAX_ZOOM_LEVEL as u8);

        let z = if start > end {
            end
        } else {
            rng.random_range(start..=end)
        };

        let z_idx = z as usize;
        let f = rng.random_range(F_MIN[z_idx]..=F_MAX[z_idx]);
        let x = rng.random_range(0..=XY_MAX[z_idx]);
        let y = rng.random_range(0..=XY_MAX[z_idx]);

        SingleId::new(z, f, x, y).expect("Failed to generate random SingleId")
    }

    #[cfg(any(test))]
    pub fn arb() -> impl Strategy<Value = Self> {
        Self::arb_within(0..=MAX_ZOOM_LEVEL as u8)
    }

    #[cfg(any(test))]
    pub fn arb_at(z: u8) -> impl Strategy<Value = Self> {
        Self::arb_within(z..=z)
    }

    #[cfg(any(test))]
    pub fn arb_within(z_range: RangeInclusive<u8>) -> impl Strategy<Value = Self> {
        z_range.prop_flat_map(|z| {
            let z_idx = z as usize;

            let f_strategy = F_MIN[z_idx]..=F_MAX[z_idx];
            let x_strategy = 0..=XY_MAX[z_idx];
            let y_strategy = 0..=XY_MAX[z_idx];

            (Just(z), f_strategy, x_strategy, y_strategy).prop_map(|(z, f, x, y)| {
                Self::new(z, f, x, y).expect("Strategy generated invalid ID")
            })
        })
    }

    #[cfg(any(test, feature = "random"))]
    /// 外部から渡された乱数生成器を使用して、特定のズームレベルの[SingleId]を作成する
    pub fn random_using<R: Rng>(rng: &mut R) -> Self {
        Self::random_within_using(rng, 0..=MAX_ZOOM_LEVEL as u8)
    }

    /// 外部から渡された乱数生成器を使用して、特定のズームレベルの[SingleId]を作成する
    #[cfg(any(test, feature = "random"))]
    pub fn random_at_using<R: Rng>(rng: &mut R, z: u8) -> Self {
        Self::random_within_using(rng, z..=z)
    }

    /// 外部から渡された乱数生成器を使用して、特定範囲の[SingleId]を作成する
    #[cfg(any(test, feature = "random"))]
    pub fn random_within_using<R: Rng>(rng: &mut R, z_range: RangeInclusive<u8>) -> Self {
        let start = *z_range.start();
        let end = (*z_range.end()).min(MAX_ZOOM_LEVEL as u8);

        let z = if start > end {
            end
        } else {
            rng.random_range(start..=end)
        };

        let z_idx = z as usize;

        // F, X, Y の範囲生成も渡された rng を使用
        let f = rng.random_range(F_MIN[z_idx]..=F_MAX[z_idx]);
        let x = rng.random_range(0..=XY_MAX[z_idx]);
        let y = rng.random_range(0..=XY_MAX[z_idx]);

        SingleId::new(z, f, x, y).expect("Failed to generate random SingleId")
    }
}
