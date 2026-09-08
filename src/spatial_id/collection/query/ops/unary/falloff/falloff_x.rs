use alloc::boxed::Box;
use alloc::vec::Vec;
use core::convert::TryFrom;
use core::fmt::Debug;
use core::marker::PhantomData;
use core::ops::{Div, Mul, Sub};

#[cfg(feature = "rayon")]
use rayon::prelude::*;

use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::query::cancellation::CancellationToken;
use crate::spatial_id::collection::query::{MergePolicy, UnaryOperator, ValueIter};
use crate::{Error, FlexId, RangeId, ZoomLevel};

use super::FalloffPattern;
use crate::spatial_id::helpers::Side;

pub struct FalloffX<P> {
    pub z: ZoomLevel,
    pub radius: u32,
    pub direction: Option<Side>,
    pub pattern: FalloffPattern,
    _marker: PhantomData<P>,
}

impl<P> FalloffX<P> {
    pub fn new<T: Into<u8>>(
        z: T,
        radius: u32,
        direction: Option<Side>,
        pattern: FalloffPattern,
    ) -> Result<Self, Error> {
        let z = ZoomLevel::new(z.into())?;
        Ok(Self {
            z,
            radius,
            direction,
            pattern,
            _marker: PhantomData,
        })
    }
}

impl<V: SafeValue + 'static, P> UnaryOperator<V> for FalloffX<P>
where
    V: Mul<Output = V> + Div<Output = V> + Sub<Output = V> + TryFrom<u32>,
    <V as TryFrom<u32>>::Error: Debug,
    P: MergePolicy<V> + Send + Sync + 'static,
{
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }

    fn run<'a>(
        &'a self,
        input: ValueIter<'a, V>,
        target: RangeId,
        token: CancellationToken,
    ) -> Result<ValueIter<'a, V>, Error> {
        if self.radius == 0 {
            return Ok(input);
        }
        let z = self.z.get();
        let radius = self.radius;

        // 減衰の到達域が近傍の入力Segment同士で重なり得るため、いったん全展開して
        // 同じ位置ごとに resolve で合成し直す。木(FlexTreeCore)は使わない —
        // このあと捨てる中間結果のために分岐構造やCOW共有を組む意味が無く、
        // ソート済み配列の方がずっと安い。
        //
        // HashMapへ直接resolveしながら畳み込む方式も試したが、FlexIdのハッシュ計算と
        // ランダムアクセスパターンのコストが、ソートのキャッシュ効率の良さを大きく上回り
        // 実測で大幅に遅く・重くなった（数百万要素規模でVec+ソートの3倍以上）。
        //
        // `target`と交差しない候補は生成した端から捨てる。`run_within`で狭い範囲だけが
        // 要求されている場合、これで無駄な候補をバッファへ積まずに済む
        // （`target`が`everything()`のときはフィルタが素通しになるだけで挙動は変わらない）。
        let mut scattered: Vec<(FlexId, V)> = Vec::new();
        let mut counter = 0u32;
        for (id, value) in input {
            token.check_amortized(&mut counter)?;
            if let Ok(iter) = id.falloff_x(z, radius, self.direction, self.pattern, &value) {
                scattered.extend(iter.filter(|(out_id, _)| out_id.intersects_range(&target)));
            }
        }

        #[cfg(feature = "rayon")]
        scattered.par_sort_unstable_by(|a, b| a.0.cmp(&b.0));
        #[cfg(not(feature = "rayon"))]
        scattered.sort_unstable_by_key(|a| a.0);

        let mut new_items = Vec::with_capacity(scattered.len());
        for chunk in scattered.chunk_by(|a, b| a.0 == b.0) {
            let id = chunk[0].0;
            let merged = chunk[1..]
                .iter()
                .fold(chunk[0].1.clone(), |acc, (_, v)| P::resolve(acc, v.clone()));
            new_items.push((id, merged));
        }

        Ok(Box::new(new_items.into_iter()))
    }

    fn inverse_bounds(&self, bounds: RangeId) -> Option<RangeId> {
        let z = self.z.get();
        let target_z = z.max(bounds.z());

        let delta = (self.radius as i64) * (1i64 << (target_z - z));
        let mut min_delta = delta;
        let mut max_delta = delta;
        if let Some(side) = self.direction {
            if side == Side::Upper {
                min_delta = 0;
            } else {
                max_delta = 0;
            }
        }

        bounds
            .x_edges_shift(target_z, -min_delta, max_delta)
            .unwrap()
    }

    fn forward_bounds(&self, bounds: RangeId) -> Option<RangeId> {
        // 逆算(出力→入力)と対になる、入力→出力の写像。片側だけに広がる場合
        // (`direction`指定あり)、逆算で伸ばした側と反対側が伸びる(鏡写し)。
        let z = self.z.get();
        let target_z = z.max(bounds.z());

        let delta = (self.radius as i64) * (1i64 << (target_z - z));
        let mut min_delta = delta;
        let mut max_delta = delta;
        if let Some(side) = self.direction {
            if side == Side::Upper {
                min_delta = 0;
            } else {
                max_delta = 0;
            }
        }

        bounds
            .x_edges_shift(target_z, -max_delta, min_delta)
            .unwrap()
    }
}
