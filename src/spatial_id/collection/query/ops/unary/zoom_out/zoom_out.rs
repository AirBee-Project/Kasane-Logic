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

/// 指定されたズームレベルまで情報を落とし、複数の子ボクセルを`MergePolicy::resolve_many` で一括マージする単項演算子。
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
    W::Value: 'static,
{
    fn as_any(&self) -> &dyn core::any::Any {
        self
    }

    fn run(&self, core: &mut W) -> Result<(), Error> {
        let mut leaves: Vec<(FlexId, W::Value)> =
            core.iter_ref().map(|(id, v)| (id, v.clone())).collect();

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

        let mut new_items = Vec::with_capacity(leaves.len());

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

        *core = W::from_flexids(new_items);

        Ok(())
    }

    fn validate(&self) -> Result<(), crate::Error> {
        Ok(())
    }

    fn commutativity_info(&self) -> CommutativityInfo {
        CommutativityInfo::none()
    }

    fn fmt_op(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "zoom_out(z={})", self.target_z.get())
    }
}
