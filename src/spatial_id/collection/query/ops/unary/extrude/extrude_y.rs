use crate::{
    FlexId,
    spatial_id::{
        collection::query::{
            merge_policy::MergePolicy,
            traits::{UnaryOperator, WorkingTree},
        },
        zoom_level::ZoomLevel,
    },
};
use alloc::boxed::Box;
use alloc::vec::Vec;
#[cfg(feature = "rayon")]
use rayon::prelude::*;

/// 任意のボクセルの現在のY座標を無視し、絶対座標の指定範囲 [start_y, end_y] に引き延ばす演算子。
pub struct ExtrudeY<P> {
    pub target_z: ZoomLevel,
    pub start_y: u32,
    pub end_y: u32,
    _marker: core::marker::PhantomData<fn() -> P>,
}

impl<P> ExtrudeY<P> {
    pub fn new(target_z: ZoomLevel, start_y: u32, end_y: u32) -> Self {
        Self {
            target_z,
            start_y,
            end_y,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<W, P> UnaryOperator<W> for ExtrudeY<P>
where
    W: WorkingTree,
    P: MergePolicy<W::Value>,
{
    fn run(&self, core: &mut W) -> Result<(), Box<dyn core::error::Error + 'static>> {
        let mut extruded: Vec<(FlexId, W::Value)> = Vec::with_capacity(core.count());

        // 元のツリーから全セルを取り出し、それぞれを引き延ばす
        for (id, v) in core.iter_ref() {
            if let Ok(iter) = id.extrude_y(self.target_z.get(), self.start_y, self.end_y) {
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
        *core = W::from_items(new_items);

        Ok(())
    }
}
