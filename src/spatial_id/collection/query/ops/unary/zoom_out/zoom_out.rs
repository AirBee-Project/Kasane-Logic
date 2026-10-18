use crate::SpatialIdTable;
use crate::spatial_id::collection::flex_tree::core::NoSummary;
use crate::spatial_id::collection::query::ops::unary::sort_by_flex_id;
use crate::spatial_id::collection::query::send_sync::SafeValue;
use crate::{
    Error, FlexId,
    spatial_id::{
        collection::query::{
            merge_policy::MergePolicy,
            traits::{QueryIter, UnaryOperator},
        },
        zoom_level::ZoomLevel,
    },
};
use alloc::vec::Vec;

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
    fn run<'a>(&'a self, input: QueryIter<'a, V>) -> Result<QueryIter<'a, V>, Error> {
        let target_z = self.target_z.get();

        if P::IS_COMMUTATIVE {
            let mut map = hashbrown::HashMap::new();
            for item in input {
                let (id, val) = item?;
                let parent = id.spatial_parent_at_zoom(target_z).unwrap();
                map.entry(parent)
                    .and_modify(|e: &mut V| *e = P::resolve(e.clone(), val.clone()))
                    .or_insert(val);
            }
            if map.is_empty() {
                return Ok(QueryIter::from_ordered(core::iter::empty()));
            }
            let mut new_items: Vec<(FlexId, V)> = map.into_iter().collect();
            sort_by_flex_id(&mut new_items);
            let table: SpatialIdTable<V, NoSummary> = new_items.into_iter().collect();
            return Ok(table.into());
        }

        let mut leaves: Vec<(FlexId, Option<V>)> = Vec::new();
        for item in input {
            let (id, v) = item?;
            leaves.push((id.spatial_parent_at_zoom(target_z).unwrap(), Some(v)));
        }

        if leaves.is_empty() {
            return Ok(QueryIter::from_ordered(core::iter::empty()));
        }

        sort_by_flex_id(&mut leaves);

        let table: SpatialIdTable<V, NoSummary> = leaves
            .chunk_by_mut(|a, b| a.0 == b.0)
            .filter_map(|chunk| {
                let parent_id = chunk[0].0;
                let merged = P::resolve_many(chunk.iter_mut().map(|(_, v)| v.take().unwrap()))?;
                Some((parent_id, merged))
            })
            .collect();

        Ok(table.into())
    }

    fn validate(&self) -> Result<(), crate::Error> {
        Ok(())
    }

    fn reverse_target(&self, bounds: crate::RangeId, out: &mut Vec<crate::RangeId>) {
        if bounds.z() > self.target_z.get() {
            out.push(bounds.spatial_parent_at_zoom(self.target_z.get()).unwrap());
        } else {
            out.push(bounds);
        }
    }
}
