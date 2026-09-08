use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::query::cancellation::CancellationToken;
use crate::spatial_id::collection::query::{MergePolicy, UnaryOperator, ValueIter};
use crate::{Error, FlexId, RangeId, spatial_id::zoom_level::ZoomLevel};

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
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }

    fn run<'a>(
        &'a self,
        input: ValueIter<'a, V>,
        _target: RangeId,
        token: CancellationToken,
    ) -> Result<ValueIter<'a, V>, Error> {
        let target_z = self.target_z.get();
        let mut counter = 0u32;
        let mut leaves: Vec<(FlexId, Option<V>)> = Vec::new();
        for (id, v) in input {
            token.check_amortized(&mut counter)?;
            leaves.push((id, Some(v)));
        }

        if leaves.is_empty() {
            return Ok(Box::new(core::iter::empty()));
        }

        // 複数の子Segmentが同じ親へ落ちるので、可換なポリシーなら順序を問わず
        // HashMapへ畳み込める。非可換なら resolve_many に順序を委ねる必要があるため、
        // 親IDでソートしてから連続run（chunk）ごとに解決する。
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
                return Ok(Box::new(new_items.into_iter()));
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
                return Ok(Box::new(new_items.into_iter()));
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

        Ok(Box::new(new_items.into_iter()))
    }

    fn inverse_bounds(&self, bounds: RangeId) -> Option<RangeId> {
        if bounds.z() > self.target_z.get() {
            Some(bounds.spatial_parent_at_zoom(self.target_z.get()).unwrap())
        } else {
            Some(bounds)
        }
    }

    fn forward_bounds(&self, input: RangeId) -> Option<RangeId> {
        // 複数の子が同じ親へ落ちるだけで、写像そのものは逆算と同じ「target_zより
        // 細かければ親を取る」で表せる。
        if input.z() > self.target_z.get() {
            Some(input.spatial_parent_at_zoom(self.target_z.get()).unwrap())
        } else {
            Some(input)
        }
    }
}
