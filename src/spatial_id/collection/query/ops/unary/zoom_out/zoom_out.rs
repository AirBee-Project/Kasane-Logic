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

/// 指定されたズームレベルまで情報を落とし（親IDへ集約し）、
/// 複数の子ボクセルを `MergePolicy::resolve_many` で一括マージする単項演算子。
pub struct ZoomOut<V, P> {
    pub target_z: ZoomLevel,
    _marker: core::marker::PhantomData<fn() -> (V, P)>,
}

impl<V, P> ZoomOut<V, P> {
    pub fn new(target_z: ZoomLevel) -> Self {
        Self {
            target_z,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<W, P> UnaryOperator<W> for ZoomOut<W::Value, P>
where
    W: WorkingTree,
    P: MergePolicy<W::Value>,
{
    fn run(&self, core: &mut W) -> Result<(), Error> {
        let mut leaves: Vec<(FlexId, W::Value)> =
            core.iter_ref().map(|(id, v)| (id, v.clone())).collect();

        // 親IDを基準にソートして、同じ親に属する子ボクセルを連続させる
        #[cfg(feature = "rayon")]
        leaves.par_sort_unstable_by(|a, b| {
            a.0.spatial_parent_at_zoom(self.target_z.get())
                .unwrap()
                .cmp(&b.0.spatial_parent_at_zoom(self.target_z.get()).unwrap())
        });

        #[cfg(not(feature = "rayon"))]
        leaves.sort_unstable_by(|a, b| {
            a.0.spatial_parent_at_zoom(self.target_z.get())
                .unwrap()
                .cmp(&b.0.spatial_parent_at_zoom(self.target_z.get()).unwrap())
        });

        // 連続する同じ親IDのグループごとに resolve_many を適用
        let mut new_items = Vec::with_capacity(leaves.len());

        // chunk_by は Rust 1.77+ で安定化（旧 group_by）
        for chunk in leaves.chunk_by(|a, b| {
            a.0.spatial_parent_at_zoom(self.target_z.get()).unwrap()
                == b.0.spatial_parent_at_zoom(self.target_z.get()).unwrap()
        }) {
            let parent_id = chunk[0]
                .0
                .spatial_parent_at_zoom(self.target_z.get())
                .unwrap();
            if let Some(merged) = P::resolve_many(chunk.iter().map(|(_, v)| v.clone())) {
                new_items.push((parent_id, merged));
            }
        }

        // 重複のない (ParentId, V) のリストからツリーを再構築
        // （すでに一意になっているため、マージ競合は発生しない）
        *core = W::from_items(new_items);

        Ok(())
    }
}
