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
use core::mem::replace;

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

        // 入力列が Z-ordered sequence なので、同一親へ縮約される要素群は必ず
        // 連続して出現し、親間の順序も保存されるため、ソート不要で逐次集約できる。
        if P::IS_COMMUTATIVE {
            let mut current: Option<(FlexId, V)> = None;
            let mut input = input.fuse();

            let iter = core::iter::from_fn(move || {
                loop {
                    match input.next() {
                        Some(Ok((id, val))) => {
                            let parent = match id.spatial_parent_at_zoom(target_z) {
                                Ok(p) => p,
                                Err(e) => return Some(Err(e)),
                            };
                            if let Some((cur_id, cur_val)) = &mut current {
                                if *cur_id == parent {
                                    *cur_val = P::resolve(cur_val.clone(), val);
                                    continue;
                                }
                                let prev = (replace(cur_id, parent), replace(cur_val, val));
                                return Some(Ok(prev));
                            } else {
                                current = Some((parent, val));
                                continue;
                            }
                        }
                        Some(Err(e)) => return Some(Err(e)),
                        None => return current.take().map(Ok),
                    }
                }
            });

            return Ok(QueryIter::from_ordered(iter));
        }

        let mut current_id: Option<FlexId> = None;
        let mut chunk: Vec<V> = Vec::new();
        let mut input = input.fuse();

        let iter = core::iter::from_fn(move || {
            loop {
                match input.next() {
                    Some(Ok((id, val))) => {
                        let parent = match id.spatial_parent_at_zoom(target_z) {
                            Ok(p) => p,
                            Err(e) => return Some(Err(e)),
                        };
                        if let Some(cur_id) = current_id {
                            if cur_id == parent {
                                chunk.push(val);
                                continue;
                            }
                            current_id = Some(parent);
                            let merged = P::resolve_many(chunk.drain(..));
                            chunk.push(val);
                            if let Some(merged) = merged {
                                return Some(Ok((cur_id, merged)));
                            }
                        } else {
                            current_id = Some(parent);
                            chunk.push(val);
                        }
                    }
                    Some(Err(e)) => return Some(Err(e)),
                    None => {
                        let id = current_id.take()?;
                        let merged = P::resolve_many(chunk.drain(..))?;
                        return Some(Ok((id, merged)));
                    }
                }
            }
        });

        Ok(QueryIter::from_ordered(iter))
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
