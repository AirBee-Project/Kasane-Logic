#[cfg(any(test, feature = "random"))]
use rand::Rng;
#[cfg(any(test, feature = "random"))]
use std::ops::RangeInclusive;

#[cfg(test)]
use proptest::prelude::*;

#[cfg(any(test, feature = "random"))]
use crate::MAX_ZOOM_LEVEL;

use crate::SingleId;
#[cfg(any(test, feature = "random"))]
use crate::{F_MAX, F_MIN, XY_MAX};

impl SingleId {
    #[cfg(any(test, feature = "random"))]
    fn pick_zoom_using<R: Rng>(rng: &mut R, z_range: RangeInclusive<u8>) -> u8 {
        let start = *z_range.start();
        let end = (*z_range.end()).min(MAX_ZOOM_LEVEL as u8);

        if start > end {
            end
        } else {
            rng.random_range(start..=end)
        }
    }

    ///ランダムな[SingleId]を作成する
    #[cfg(any(test, feature = "random"))]
    pub fn random() -> Self {
        let mut rng = rand::rng();
        Self::random_using(&mut rng)
    }

    ///特定のズームレベルにおいて、ランダムな[SingleId]を作成する
    #[cfg(any(test, feature = "random"))]
    pub fn random_at(z: u8) -> Self {
        let mut rng = rand::rng();
        Self::random_at_using(&mut rng, z)
    }

    ///特定のズームレベル間において、ランダムな[SingleId]を作成する
    #[cfg(any(test, feature = "random"))]
    pub fn random_within(z_range: RangeInclusive<u8>) -> Self {
        let mut rng = rand::rng();
        Self::random_within_using(&mut rng, z_range)
    }

    /// `proptest` 用に、全ズーム範囲から [`SingleId`] を生成する戦略を返します。
    #[cfg(test)]
    pub fn arb() -> impl Strategy<Value = Self> {
        Self::arb_within(0..=MAX_ZOOM_LEVEL as u8)
    }

    /// `proptest` 用に、指定ズームの [`SingleId`] を生成する戦略を返します。
    #[cfg(test)]
    pub fn arb_at(z: u8) -> impl Strategy<Value = Self> {
        Self::arb_within(z..=z)
    }

    /// `proptest` 用に、指定ズーム範囲の [`SingleId`] を生成する戦略を返します。
    ///
    /// `z_range` の終端は `MAX_ZOOM_LEVEL` でクリップされ、`start > end` の場合は `end` のみを使います。
    #[cfg(test)]
    pub fn arb_within(z_range: RangeInclusive<u8>) -> impl Strategy<Value = Self> {
        let start = *z_range.start();
        let end = (*z_range.end()).min(MAX_ZOOM_LEVEL as u8);
        let (start, end) = if start > end {
            (end, end)
        } else {
            (start, end)
        };

        (start..=end).prop_flat_map(|z| {
            let z_idx = z as usize;

            let f_strategy = F_MIN[z_idx]..=F_MAX[z_idx];
            let x_strategy = 0..=XY_MAX[z_idx];
            let y_strategy = 0..=XY_MAX[z_idx];

            (Just(z), f_strategy, x_strategy, y_strategy).prop_map(|(z, f, x, y)| {
                Self::new(z, f, x, y).expect("Strategy generated invalid ID")
            })
        })
    }

    /// 外部から渡された乱数生成器を使って、全ズーム範囲からランダムな [`SingleId`] を生成します。
    #[cfg(any(test, feature = "random"))]
    pub fn random_using<R: Rng>(rng: &mut R) -> Self {
        Self::random_within_using(rng, 0..=MAX_ZOOM_LEVEL as u8)
    }

    /// 外部から渡された乱数生成器を使って、指定ズームのランダムな [`SingleId`] を生成します。
    #[cfg(any(test, feature = "random"))]
    pub fn random_at_using<R: Rng>(rng: &mut R, z: u8) -> Self {
        Self::random_within_using(rng, z..=z)
    }

    /// 外部から渡された乱数生成器を使って、指定ズーム範囲のランダムな [`SingleId`] を生成します。
    ///
    /// `z_range` の終端は `MAX_ZOOM_LEVEL` でクリップされ、`start > end` の場合は `end` を採用します。
    #[cfg(any(test, feature = "random"))]
    pub fn random_within_using<R: Rng>(rng: &mut R, z_range: RangeInclusive<u8>) -> Self {
        let z = Self::pick_zoom_using(rng, z_range);

        let z_idx = z as usize;

        // F, X, Y の範囲生成も渡された rng を使用
        let f = rng.random_range(F_MIN[z_idx]..=F_MAX[z_idx]);
        let x = rng.random_range(0..=XY_MAX[z_idx]);
        let y = rng.random_range(0..=XY_MAX[z_idx]);

        SingleId::new(z, f, x, y).expect("Failed to generate random SingleId")
    }
}
