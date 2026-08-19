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

/// 指定されたズームレベルまで情報を落とす演算子。
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

impl<V: SafeValue + 'static, P> UnaryOperator<V> for ZoomOut<V, P>
where
    P: MergePolicy<V>,
{
    fn as_any(&self) -> &dyn core::any::Any {
        self
    }

    fn run(&self, core: &mut WorkingTree<V>) -> Result<(), Error> {
        let target_z = self.target_z.get();
        let old_tree = core::mem::take(core);
        let mut leaves: Vec<(FlexId, Option<V>)> =
            old_tree.into_iter().map(|(id, v)| (id, Some(v))).collect();

        if leaves.is_empty() {
            return Ok(());
        }

        #[cfg(feature = "rayon")]
        {
            if P::IS_COMMUTATIVE {
                let mut map = hashbrown::HashMap::with_capacity(leaves.len());
                for (id, v) in leaves {
                    let parent = id.spatial_parent_at_zoom(target_z).unwrap();
                    let val = v.unwrap();
                    map.entry(parent)
                        .and_modify(|e: &mut V| *e = P::resolve(e.clone(), val.clone()))
                        .or_insert(val);
                }

                let mut new_items: Vec<(FlexId, V)> = map.into_iter().collect();
                new_items.par_sort_unstable_by_key(|a| a.0);
                *core = new_items.into_iter().collect();
                return Ok(());
            }

            leaves.par_iter_mut().for_each(|(id, _)| {
                *id = id.spatial_parent_at_zoom(target_z).unwrap();
            });
            leaves.par_sort_unstable_by_key(|a| a.0);
        }

        #[cfg(not(feature = "rayon"))]
        {
            if P::IS_COMMUTATIVE {
                let mut map = hashbrown::HashMap::with_capacity(leaves.len());
                for (id, v) in leaves {
                    let parent = id.spatial_parent_at_zoom(target_z).unwrap();
                    let val = v.unwrap();
                    map.entry(parent)
                        .and_modify(|e: &mut V| *e = P::resolve(e.clone(), val.clone()))
                        .or_insert(val);
                }

                let mut new_items: Vec<(FlexId, V)> = map.into_iter().collect();
                new_items.sort_unstable_by_key(|a| a.0);
                *core = new_items.into_iter().collect();
                return Ok(());
            }

            for (id, _) in leaves.iter_mut() {
                *id = id.spatial_parent_at_zoom(target_z).unwrap();
            }
            leaves.sort_unstable_by_key(|a| a.0);
        }

        #[cfg(feature = "rayon")]
        let new_items: Vec<(FlexId, V)> = {
            leaves
                .par_chunk_by_mut(|a, b| a.0 == b.0)
                .filter_map(|chunk| {
                    let parent_id = chunk[0].0;
                    let merged = P::resolve_many(chunk.iter_mut().map(|(_, v)| v.take().unwrap()))?;
                    Some((parent_id, merged))
                })
                .collect()
        };

        #[cfg(not(feature = "rayon"))]
        let new_items: Vec<(FlexId, V)> = {
            leaves
                .chunk_by_mut(|a, b| a.0 == b.0)
                .filter_map(|chunk| {
                    let parent_id = chunk[0].0;
                    let merged = P::resolve_many(chunk.iter_mut().map(|(_, v)| v.take().unwrap()))?;
                    Some((parent_id, merged))
                })
                .collect()
        };

        *core = new_items.into_iter().collect();

        Ok(())
    }

    fn validate(&self) -> Result<(), crate::Error> {
        Ok(())
    }

    fn inverse_bounds(&self, bounds: crate::RangeId) -> Option<crate::RangeId> {
        if bounds.z() > self.target_z.get() {
            Some(bounds.spatial_parent_at_zoom(self.target_z.get()).unwrap())
        } else {
            Some(bounds)
        }
    }

    fn commutativity_info(&self) -> CommutativityInfo {
        CommutativityInfo::None
    }

    fn grid_zoom(&self) -> Option<crate::ZoomLevel> {
        Some(self.target_z)
    }

    #[allow(private_interfaces)]
    fn apply_to_grid(
        &self,
        grid: &mut crate::spatial_id::collection::query::grid::UniformGrid<V>,
        token: &crate::CancellationToken,
    ) -> Result<crate::spatial_id::collection::query::grid::Applied, Error> {
        if !P::IS_COMMUTATIVE {
            return Ok(crate::spatial_id::collection::query::grid::Applied::Unsupported);
        }
        grid.zoom_out::<P>(self.target_z, token)?;
        Ok(crate::spatial_id::collection::query::grid::Applied::Done)
    }

    fn fmt_op(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "zoom_out(z={})", self.target_z.get())
    }
}
