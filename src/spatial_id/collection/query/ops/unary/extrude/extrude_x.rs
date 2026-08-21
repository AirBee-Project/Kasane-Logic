use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::query::execution::group_commutative::types::CommutativityInfo;
use crate::spatial_id::collection::query::working::WorkingTree;
use crate::{
    Error, FlexId,
    spatial_id::{
        collection::query::{merge_policy::MergePolicy, traits::UnaryOperator},
        zoom_level::ZoomLevel,
    },
};
use alloc::vec::Vec;
#[cfg(feature = "rayon")]
use rayon::prelude::*;

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

impl<V: SafeValue, P> UnaryOperator<V> for ExtrudeX<P>
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

    fn run(&self, core: &mut WorkingTree<V>) -> Result<(), Error> {
        let expected_cap = libm::ceil(core.core().count() as f64 * self.expansion_ratio()) as usize;
        let mut extruded: Vec<(FlexId, V)> = Vec::with_capacity(expected_cap);

        // 元のツリーから全Segmentを取り出し、それぞれを引き延ばす
        for (id, v) in core.core().iter_ref() {
            if let Ok(iter) = id.extrude_x(self.target_z.get(), self.start_x, self.end_x) {
                for new_id in iter {
                    extruded.push((new_id, v.clone()));
                }
            }
        }

        // 重複や競合を解決するため、IDでソートする
        #[cfg(feature = "rayon")]
        extruded.par_sort_unstable_by(|a, b| a.0.cmp(&b.0));

        #[cfg(not(feature = "rayon"))]
        extruded.sort_unstable_by_key(|a| a.0);

        // 連続する同じIDのグループごとに resolve_many を適用
        let mut new_items = Vec::with_capacity(extruded.len());
        for chunk in extruded.chunk_by(|a, b| a.0 == b.0) {
            let id = chunk[0].0;
            if let Some(merged) = P::resolve_many(chunk.iter().map(|(_, v)| v.clone())) {
                new_items.push((id, merged));
            }
        }

        // 重複のない (FlexId, V) のリストからツリーを再構築
        *core = new_items.into_iter().collect();

        Ok(())
    }

    fn forward_map(
        &self,
        id: crate::FlexId,
        value: V,
        out: &mut alloc::vec::Vec<(crate::FlexId, V)>,
    ) -> Result<(), crate::Error> {
        for new_id in id.extrude_x(self.target_z.get(), self.start_x, self.end_x)? {
            out.push((new_id, value.clone()));
        }
        Ok(())
    }

    fn collision_merge(&self) -> Option<fn(&mut alloc::vec::Vec<(crate::FlexId, V)>)> {
        Some(
            crate::spatial_id::collection::query::execution::composed_chain::merge_sorted_vec::<V, P>,
        )
    }

    fn tree_resolver(&self) -> Option<fn(&V, &V) -> V> {
        Some(|a: &V, b: &V| P::resolve(a.clone(), b.clone()))
    }

    fn commutativity_info(&self) -> CommutativityInfo {
        if !P::IS_COMMUTATIVE {
            return CommutativityInfo::None;
        }
        CommutativityInfo::AbsoluteTarget {
            axis: crate::spatial_id::collection::query::execution::group_commutative::types::TargetAxis::X,
            policy: Some(core::any::TypeId::of::<P>()),
        }
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }

    fn inverse_bounds(&self, mut bounds: crate::RangeId) -> Option<crate::RangeId> {
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
        let has_overlap = match crate::RangeId::split_wrapped_range(
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

        let xy_max = crate::ZoomLevel::new(bounds_z).unwrap().xy_max();
        bounds.set_x([0, xy_max]).unwrap();
        Some(bounds)
    }

    fn expansion_ratio(&self) -> f64 {
        self.start_x.abs_diff(self.end_x) as f64 + 1.0
    }

    fn fmt_op(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "extrude_x(z={}, x=[{}, {}], {})",
            self.target_z.get(),
            self.start_x,
            self.end_x,
            P::NAME
        )
    }
}
