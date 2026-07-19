use crate::{
    FlexId, FlexTreeCore,
    spatial_id::{
        collection::query::{merge_policy::MergePolicy, traits::UnaryOperator},
        zoom_level::ZoomLevel,
    },
};
use alloc::boxed::Box;
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

impl<V, P> UnaryOperator<V> for ExtrudeF<P>
where
    V: crate::spatial_id::collection::flex_tree::core::SafeValue,
    P: MergePolicy<V>,
{
    fn run(&self, core: &mut FlexTreeCore<V>) -> Result<(), Box<dyn core::error::Error + 'static>> {
        let mut extruded: Vec<(FlexId, V)> = Vec::with_capacity(core.count());

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
        #[cfg(feature = "rayon")]
        {
            *core = FlexTreeCore::par_build_vec(new_items);
        }
        #[cfg(not(feature = "rayon"))]
        {
            let mut new_core = FlexTreeCore::new();
            for (id, val) in new_items {
                new_core.insert(id, val);
            }
            *core = new_core;
        }

        Ok(())
    }
}
