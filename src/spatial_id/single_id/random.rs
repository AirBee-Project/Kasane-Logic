#[cfg(any(test, feature = "random"))]
use crate::SingleId;
#[cfg(any(test, feature = "random"))]
use crate::ZoomLevel;
#[cfg(any(test, feature = "random"))]
use core::ops::RangeInclusive;
#[cfg(test)]
use proptest::prelude::*;
#[cfg(any(test, feature = "random"))]
use rand::{Rng, RngExt};
#[cfg(any(test, feature = "random"))]
impl SingleId {
    #[cfg(any(test, feature = "random"))]
    fn pick_zoom_using<R: Rng>(rng: &mut R, z_range: RangeInclusive<u8>) -> u8 {
        let start = *z_range.start();
        let end = (*z_range.end()).min(ZoomLevel::MAX.get());

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
        Self::arb_within(0..=ZoomLevel::MAX.get())
    }

    /// `proptest` 用に、指定ズームの [`SingleId`] を生成する戦略を返します。
    #[cfg(test)]
    pub fn arb_at(z: u8) -> impl Strategy<Value = Self> {
        Self::arb_within(z..=z)
    }

    /// `proptest` 用に、指定ズーム範囲の [`SingleId`] を生成する戦略を返します。
    ///
    /// `z_range` の終端は `ZoomLevel::MAX` でクリップされ、`start > end` の場合は `end` のみを使います。
    #[cfg(test)]
    pub fn arb_within(z_range: RangeInclusive<u8>) -> impl Strategy<Value = Self> {
        let start = *z_range.start();
        let end = (*z_range.end()).min(ZoomLevel::MAX.get());
        let (start, end) = if start > end {
            (end, end)
        } else {
            (start, end)
        };

        (start..=end).prop_flat_map(|z| {
            let z_idx = z as usize;

            let f_strategy = ZoomLevel::new(z_idx as u8).unwrap().f_min()
                ..=ZoomLevel::new(z_idx as u8).unwrap().f_max();
            let x_strategy = 0..=ZoomLevel::new(z_idx as u8).unwrap().xy_max();
            let y_strategy = 0..=ZoomLevel::new(z_idx as u8).unwrap().xy_max();

            (Just(z), f_strategy, x_strategy, y_strategy).prop_map(|(z, f, x, y)| {
                Self::new(z, f, x, y).expect("Strategy generated invalid ID")
            })
        })
    }

    /// 外部から渡された乱数生成器を使って、全ズーム範囲からランダムな [`SingleId`] を生成します。
    #[cfg(any(test, feature = "random"))]
    pub fn random_using<R: Rng>(rng: &mut R) -> Self {
        Self::random_within_using(rng, 0..=ZoomLevel::MAX.get())
    }

    /// 外部から渡された乱数生成器を使って、指定ズームのランダムな [`SingleId`] を生成します。
    #[cfg(any(test, feature = "random"))]
    pub fn random_at_using<R: Rng>(rng: &mut R, z: u8) -> Self {
        Self::random_within_using(rng, z..=z)
    }

    /// 外部から渡された乱数生成器を使って、指定ズーム範囲のランダムな [`SingleId`] を生成します。
    ///
    /// `z_range` の終端は `ZoomLevel::MAX` でクリップされ、`start > end` の場合は `end` を採用します。
    #[cfg(any(test, feature = "random"))]
    pub fn random_within_using<R: Rng>(rng: &mut R, z_range: RangeInclusive<u8>) -> Self {
        let z = Self::pick_zoom_using(rng, z_range);

        let z_idx = z as usize;

        // F, X, Y の範囲生成も渡された rng を使用
        let f = rng.random_range(
            ZoomLevel::new(z_idx as u8).unwrap().f_min()
                ..=ZoomLevel::new(z_idx as u8).unwrap().f_max(),
        );
        let x = rng.random_range(0..=ZoomLevel::new(z_idx as u8).unwrap().xy_max());
        let y = rng.random_range(0..=ZoomLevel::new(z_idx as u8).unwrap().xy_max());

        SingleId::new(z, f, x, y).expect("Failed to generate random SingleId")
    }
}
