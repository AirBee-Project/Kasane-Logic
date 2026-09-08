use alloc::boxed::Box;
use alloc::vec::Vec;
#[cfg(feature = "rayon")]
use rayon::prelude::*;

use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::query::cancellation::CancellationToken;
use crate::spatial_id::collection::query::{MergePolicy, UnaryOperator, ValueIter};
use crate::{Error, FlexId, RangeId, ZoomLevel};

/// 任意のボクSegmentの現在のX座標を無視し、絶対座標の指定範囲 [start_x, end_x] に引き延ばす演算子。
pub struct ExtrudeX<P> {
    pub target_z: ZoomLevel,
    pub start_x: u32,
    pub end_x: u32,
    _marker: core::marker::PhantomData<fn() -> P>,
}

impl<P> ExtrudeX<P> {
    pub fn new(target_z: ZoomLevel, start_x: u32, end_x: u32) -> Self {
        Self {
            target_z,
            start_x,
            end_x,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<V: SafeValue + 'static, P> UnaryOperator<V> for ExtrudeX<P>
where
    P: MergePolicy<V>,
{
    fn validate(&self) -> Result<(), Error> {
        let z = self.target_z.get();
        let zl = ZoomLevel::new(z)?;
        zl.check_x(self.start_x)?;
        zl.check_x(self.end_x)?;
        Ok(())
    }

    fn run<'a>(
        &'a self,
        input: ValueIter<'a, V>,
        target: RangeId,
        token: CancellationToken,
    ) -> Result<ValueIter<'a, V>, Error> {
        let target_z = self.target_z.get();

        // 引き延ばした先の絶対座標は入力Segmentどうしで重なり得るため、いったん全展開して
        // 同じ移動先ごとに resolve_many で合成し直す。extrudeは1入力が[start,end]全体へ
        // 広がるので、`target`と交差しない候補を生成した端から捨てる効果が特に大きい
        // （`target`が`everything()`のときはフィルタが素通しになるだけで挙動は変わらない）。
        let mut extruded: Vec<(FlexId, V)> = Vec::new();
        let mut counter = 0u32;
        for (id, v) in input {
            token.check_amortized(&mut counter)?;
            if let Ok(iter) = id.extrude_x(target_z, self.start_x, self.end_x) {
                extruded.extend(
                    iter.filter(|new_id| new_id.intersects_range(&target))
                        .map(|new_id| (new_id, v.clone())),
                );
            }
        }

        #[cfg(feature = "rayon")]
        extruded.par_sort_unstable_by(|a, b| a.0.cmp(&b.0));
        #[cfg(not(feature = "rayon"))]
        extruded.sort_unstable_by_key(|a| a.0);

        let mut new_items = Vec::with_capacity(extruded.len());
        for chunk in extruded.chunk_by(|a, b| a.0 == b.0) {
            let id = chunk[0].0;
            if let Some(merged) = P::resolve_many(chunk.iter().map(|(_, v)| v.clone())) {
                new_items.push((id, merged));
            }
        }

        Ok(Box::new(new_items.into_iter()))
    }

    fn inverse_bounds(&self, mut bounds: RangeId) -> Option<RangeId> {
        let target_z = self.target_z.get();
        let bounds_z = bounds.z();
        let max_z = target_z.max(bounds_z);

        let scale_t = max_z - target_z;

        let target_min_max_z = (self.start_x as i64) * (1i64 << scale_t);
        let target_max_max_z = ((self.end_x as i64) + 1) * (1i64 << scale_t) - 1;

        let (bounds_min_max_z, bounds_max_max_z) = bounds.x_fine_range(max_z);
        let bounds_min_max_z = bounds_min_max_z as i64;
        let bounds_max_max_z = bounds_max_max_z as i64;

        let overlaps = |min: i64, max: i64| !(target_max_max_z < min || max < target_min_max_z);

        // bounds のx軸が折り返し（`bounds.x()[0] > bounds.x()[1]`）の場合、x_fine_range は
        // それを考慮せず min > max のまま返す。2つの非折り返し区間に分解し、どちらかが
        // target と重なれば良い（単純な区間判定は折り返しには使えないため）。
        let has_overlap = match RangeId::split_wrapped_range(
            bounds_min_max_z,
            bounds_max_max_z,
            (1i64 << max_z) - 1,
        ) {
            Some([(a_min, a_max), (b_min, b_max)]) => {
                overlaps(a_min, a_max) || overlaps(b_min, b_max)
            }
            None => overlaps(bounds_min_max_z, bounds_max_max_z),
        };

        if !has_overlap {
            return None;
        }

        let xy_max = ZoomLevel::new(bounds_z).unwrap().xy_max();
        bounds.set_x([0, xy_max]).unwrap();
        Some(bounds)
    }
}
