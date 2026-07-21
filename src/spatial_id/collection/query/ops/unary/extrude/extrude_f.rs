use crate::spatial_id::collection::query::execution::group_commutative::types::CommutativityInfo;
use crate::{
    Error, FlexId,
    spatial_id::{
        collection::query::{
            merge_policy::MergePolicy,
            traits::{UnaryOperator, WorkingTree},
        },
        zoom_level::ZoomLevel,
    },
};
use alloc::vec::Vec;
#[cfg(feature = "rayon")]
use rayon::prelude::*;

/// 任意のボクセルの現在のF座標を無視し、絶対座標の指定範囲 [start_f, end_f] に引き延ばす演算子。
pub struct ExtrudeF<P> {
    pub target_z: ZoomLevel,
    pub start_f: i32,
    pub end_f: i32,
    _marker: core::marker::PhantomData<fn() -> P>,
}

impl<P> ExtrudeF<P> {
    pub fn new(target_z: ZoomLevel, start_f: i32, end_f: i32) -> Self {
        Self {
            target_z,
            start_f,
            end_f,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<W, P> UnaryOperator<W> for ExtrudeF<P>
where
    W: WorkingTree + 'static,
    P: MergePolicy<W::Value>,
{
    fn validate(&self) -> Result<(), Error> {
        let z = self.target_z.get();
        let zl = ZoomLevel::new(z)?;
        zl.check_f(self.start_f)?;
        zl.check_f(self.end_f)?;
        Ok(())
    }

    fn run(&self, core: &mut W) -> Result<(), Error> {
        let mut extruded: Vec<(FlexId, W::Value)> = Vec::with_capacity(core.count());

        // 元のツリーから全セルを取り出し、それぞれを引き延ばす
        for (id, v) in core.iter_ref() {
            if let Ok(iter) = id.extrude_f(self.target_z.get(), self.start_f, self.end_f) {
                for new_id in iter {
                    extruded.push((new_id, v.clone()));
                }
            }
        }

        // 重複や競合を解決するため、IDでソートする
        #[cfg(feature = "rayon")]
        extruded.par_sort_unstable_by(|a, b| a.0.cmp(&b.0));

        #[cfg(not(feature = "rayon"))]
        extruded.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        // 連続する同じIDのグループごとに resolve_many を適用
        let mut new_items = Vec::with_capacity(extruded.len());
        for chunk in extruded.chunk_by(|a, b| a.0 == b.0) {
            let id = chunk[0].0.clone();
            if let Some(merged) = P::resolve_many(chunk.iter().map(|(_, v)| v.clone())) {
                new_items.push((id, merged));
            }
        }

        // 重複のない (FlexId, V) のリストからツリーを再構築
        *core = W::from_flexids(new_items);

        Ok(())
    }

    fn commutativity_info(&self) -> CommutativityInfo {
        CommutativityInfo::absolute_target::<P>(
            crate::spatial_id::collection::query::execution::group_commutative::types::TargetAxis::F,
            P::IS_COMMUTATIVE,
        )
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }

    fn inverse_bounds(&self, mut bounds: crate::RangeId) -> alloc::vec::Vec<crate::RangeId> {
        let target_z = self.target_z.get();
        let bounds_z = bounds.z();
        let max_z = target_z.max(bounds_z);

        let scale_t = max_z - target_z;
        let scale_b = max_z - bounds_z;

        let target_min_max_z = (self.start_f as i64) * (1i64 << scale_t);
        let target_max_max_z = ((self.end_f as i64) + 1) * (1i64 << scale_t) - 1;

        let bounds_min_max_z = (bounds.f()[0] as i64) * (1i64 << scale_b);
        let bounds_max_max_z = ((bounds.f()[1] as i64) + 1) * (1i64 << scale_b) - 1;

        if target_max_max_z < bounds_min_max_z || bounds_max_max_z < target_min_max_z {
            return alloc::vec![];
        }

        let max_z_obj = crate::ZoomLevel::new(bounds_z).unwrap();
        bounds
            .set_f([max_z_obj.f_min(), max_z_obj.f_max()])
            .unwrap();
        alloc::vec![bounds]
    }

    fn expansion_ratio(&self) -> f32 {
        self.start_f.abs_diff(self.end_f) as f32 + 1.0
    }

    fn fmt_op(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "extrude_f(z={}, f=[{}, {}], {})",
            self.target_z.get(),
            self.start_f,
            self.end_f,
            P::NAME
        )
    }
}
