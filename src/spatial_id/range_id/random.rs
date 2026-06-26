#[cfg(any(test, feature = "random"))]
use core::ops::RangeInclusive;
#[cfg(test)]
use proptest::prelude::Strategy;
#[cfg(any(test, feature = "random"))]
use rand::{Rng, RngExt};

use crate::RangeId;
#[cfg(any(test, feature = "random"))]
use crate::ZoomLevel;

impl RangeId {
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

    /// 外部から渡された乱数生成器を使って、全ズーム範囲からランダムな [`RangeId`] を生成します。
    #[cfg(any(test, feature = "random"))]
    pub fn random_using<R: Rng>(rng: &mut R) -> Self {
        Self::random_within_using(rng, 0..=ZoomLevel::MAX.get())
    }

    /// 外部から渡された乱数生成器を使用して、指定したズームレベルでランダムにRangeIdを生成
    #[cfg(any(test, feature = "random"))]
    pub fn random_at_using<R: Rng>(rng: &mut R, z: u8) -> Self {
        Self::random_within_using(rng, z..=z)
    }

    /// 外部から渡された乱数生成器を使用して、指定したズームレベル範囲内でランダムにRangeIdを生成
    #[cfg(any(test, feature = "random"))]
    pub fn random_within_using<R: Rng>(rng: &mut R, z_range: RangeInclusive<u8>) -> Self {
        let z = Self::pick_zoom_using(rng, z_range);
        let z_idx = z as usize;

        let f_min = ZoomLevel::new(z_idx as u8).unwrap().f_min();
        let f_max = ZoomLevel::new(z_idx as u8).unwrap().f_max();
        let xy_max = ZoomLevel::new(z_idx as u8).unwrap().xy_max();

        // 範囲の両端をランダムに生成
        let f1 = rng.random_range(f_min..=f_max);
        let f2 = rng.random_range(f_min..=f_max);

        let x1 = rng.random_range(0..=xy_max);
        let x2 = rng.random_range(0..=xy_max);

        let y1 = rng.random_range(0..=xy_max);
        let y2 = rng.random_range(0..=xy_max);

        // RangeId::new 内部で min/max の入れ替え等は処理される前提
        RangeId::new(z, [f1, f2], [x1, x2], [y1, y2])
            .expect("Generated parameters should be always valid")
    }

    /// 全空間（Z=0〜MAX）からランダムにRangeIdを生成
    #[cfg(any(test, feature = "random"))]
    pub fn random() -> Self {
        let mut rng = rand::rng();
        Self::random_using(&mut rng)
    }

    /// 指定したズームレベルでランダムにRangeIdを生成
    #[cfg(any(test, feature = "random"))]
    pub fn random_at(z: u8) -> Self {
        let mut rng = rand::rng();
        Self::random_at_using(&mut rng, z)
    }

    /// 指定ズーム範囲からランダムな [`RangeId`] を生成します。
    ///
    /// 内部で `rand::rng()` を作って [`random_within_using`](Self::random_within_using) に委譲します。
    #[cfg(any(test, feature = "random"))]
    pub fn random_within(z_range: RangeInclusive<u8>) -> Self {
        let mut rng = rand::rng();
        Self::random_within_using(&mut rng, z_range)
    }

    /// `proptest` 用に、全ズーム範囲から [`RangeId`] を生成する戦略を返します。
    #[cfg(test)]
    pub fn arb() -> impl Strategy<Value = Self> {
        Self::arb_within(0..=ZoomLevel::MAX.get())
    }

    /// `proptest` 用に、指定ズームの [`RangeId`] を生成する戦略を返します。
    #[cfg(test)]
    pub fn arb_at(z: u8) -> impl Strategy<Value = Self> {
        Self::arb_within(z..=z)
    }

    /// `proptest` 用に、指定ズーム範囲の [`RangeId`] を生成する戦略を返します。
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
            use proptest::prelude::Just;

            let z_idx = z as usize;

            let f_min = ZoomLevel::new(z_idx as u8).unwrap().f_min();
            let f_max = ZoomLevel::new(z_idx as u8).unwrap().f_max();
            let xy_max = ZoomLevel::new(z_idx as u8).unwrap().xy_max();

            let f_strat = (f_min..=f_max, f_min..=f_max);
            let x_strat = (0..=xy_max, 0..=xy_max);
            let y_strat = (0..=xy_max, 0..=xy_max);

            (Just(z), f_strat, x_strat, y_strat).prop_map(
                move |(z, (f1, f2), (x1, x2), (y1, y2))| {
                    RangeId::new(z, [f1, f2], [x1, x2], [y1, y2])
                        .expect("Generated parameters should be always valid")
                },
            )
        })
    }
}
