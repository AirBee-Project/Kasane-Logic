use alloc::vec::Vec;

use super::{FlexTreeCore, node::Node, ptr::SharedNode, split_child_id};
use crate::{FlexId, Side};

impl<V> FlexTreeCore<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue,
{
    /// このFlexTreeをシャードのために分割する際に、どこを切り出せば良いのかを判断する関数。
    pub(crate) fn balanced_cut(&self) -> Option<FlexId> {
        let lower = self.lower_root.leaf_count();
        let upper = self.upper_root.leaf_count();
        let total = lower + upper;
        if total == 0 {
            return None;
        }

        let half = total as i64 / 2;
        let closeness = |count: usize| (count as i64 - half).unsigned_abs();

        let (heavy_root, heavy_id, light_id, light_count) = if lower >= upper {
            (
                &self.lower_root,
                FlexId::LOWER_MAX,
                FlexId::UPPER_MAX,
                upper,
            )
        } else {
            (
                &self.upper_root,
                FlexId::UPPER_MAX,
                FlexId::LOWER_MAX,
                lower,
            )
        };

        let mut best_region = light_id;
        let mut best_count = light_count;

        let mut node = heavy_root.clone();
        let mut id = heavy_id;
        loop {
            let branch = match &*node {
                Node::Branch {
                    level,
                    lower_child,
                    upper_child,
                    ..
                } => Some((
                    *level,
                    lower_child.leaf_count(),
                    upper_child.leaf_count(),
                    lower_child.clone(),
                    upper_child.clone(),
                )),
                Node::Leaf { .. } => None,
            };
            let Some((level, l, u, lower_child, upper_child)) = branch else {
                break;
            };

            let axis = Node::<V>::axis(level);
            let lower_id = split_child_id(&id, axis, Side::Lower);
            let upper_id = split_child_id(&id, axis, Side::Upper);

            if closeness(l) < closeness(best_count) {
                best_region = lower_id.clone();
                best_count = l;
            }
            if closeness(u) < closeness(best_count) {
                best_region = upper_id.clone();
                best_count = u;
            }

            // 偏りの原因は重い子の中にある → まだ過半なら重い子へ降りる。
            let (heavy_child, heavy_child_id, heavy_count) = if l >= u {
                (lower_child, lower_id, l)
            } else {
                (upper_child, upper_id, u)
            };
            if heavy_count as i64 > half {
                node = heavy_child;
                id = heavy_child_id;
            } else {
                break;
            }
        }

        Some(best_region)
    }

    /// この[`FlexTreeCore`]をシャード分割すべきかを判定する。保持する[FlexId]数が `max_flex_id_count` を超えていれば `true`を返す。[FlexId]の個数はキャッシュされているため高速に動作する。
    pub fn should_split_shard(&self, max_flex_id_count: usize) -> bool {
        self.count() > max_flex_id_count
    }

    /// [`FlexTreeCore`]を互いに素なシャードへ分割する。この時、シャード1つあたりの中身の[FlexId]の個数は`max_flex_id_count` 以下になる。分割の必要がなければ、自分自身を返す。
    pub fn split_shard(&self, max_flex_id_count: usize) -> Vec<Self> {
        let mut result = Vec::new();
        let mut pending = alloc::vec![self.clone()];

        while let Some(piece) = pending.pop() {
            // 閾値以下、または分割不能（1要素以下）ならそのまま確定
            if piece.count() <= max_flex_id_count || piece.count() < 2 {
                result.push(piece);
                continue;
            }

            let Some(region) = piece.balanced_cut() else {
                result.push(piece);
                continue;
            };

            let sub_pieces: Vec<Self> = piece
                .shard_regions(region)
                .into_iter()
                .map(|piece_region| piece.extract_region(piece_region))
                .collect();
            pending.extend(sub_pieces);
        }

        result
    }

    pub(crate) fn shard_regions(&self, region: FlexId) -> Vec<FlexId> {
        let mut regions = alloc::vec![region.clone()];

        // R が属するルートと、反対側ルート。
        let in_lower = region.f_index() < 0;
        let (region_root, region_root_id, other_root, other_root_id) = if in_lower {
            (
                &self.lower_root,
                FlexId::LOWER_MAX,
                &self.upper_root,
                FlexId::UPPER_MAX,
            )
        } else {
            (
                &self.upper_root,
                FlexId::UPPER_MAX,
                &self.lower_root,
                FlexId::LOWER_MAX,
            )
        };

        if other_root.leaf_count() > 0 {
            regions.push(other_root_id);
        }

        let mut node = region_root.clone();
        let mut id = region_root_id;
        while id != region {
            let next = match &*node {
                Node::Branch {
                    level,
                    lower_child,
                    upper_child,
                    ..
                } => {
                    let axis = Node::<V>::axis(*level);
                    let lower_id = split_child_id(&id, axis, Side::Lower);
                    let upper_id = split_child_id(&id, axis, Side::Upper);
                    if lower_id.intersection(&region).is_some() {
                        if upper_child.leaf_count() > 0 {
                            regions.push(upper_id);
                        }
                        (lower_child.clone(), lower_id)
                    } else {
                        if lower_child.leaf_count() > 0 {
                            regions.push(lower_id);
                        }
                        (upper_child.clone(), upper_id)
                    }
                }
                Node::Leaf { .. } => break,
            };
            node = next.0;
            id = next.1;
        }

        regions
    }

    fn extract_region(&self, region: FlexId) -> Self {
        let in_lower = region.f_index() < 0;

        let mut piece = self.clone();
        {
            let (root, root_id) = if in_lower {
                (&mut piece.lower_root, FlexId::LOWER_MAX)
            } else {
                (&mut piece.upper_root, FlexId::UPPER_MAX)
            };
            Self::prune_path(root, root_id, &region, true, &self.empty_leaf);
        }
        if in_lower {
            piece.upper_root = self.empty_leaf.clone();
        } else {
            piece.lower_root = self.empty_leaf.clone();
        }
        piece.shard = Some(region);
        piece
    }

    fn prune_path(
        node: &mut SharedNode<Node<V>>,
        current_id: FlexId,
        region: &FlexId,
        keep: bool,
        empty_leaf: &SharedNode<Node<V>>,
    ) {
        if &current_id == region {
            if !keep {
                *node = empty_leaf.clone();
            }
            return;
        }

        let mut_node = SharedNode::make_mut(node);
        if let Node::Branch {
            level,
            lower_child,
            upper_child,
            leaf_count,
            max_zoom,
        } = mut_node
        {
            let axis = Node::<V>::axis(*level);
            let lower_id = split_child_id(&current_id, axis, Side::Lower);
            let upper_id = split_child_id(&current_id, axis, Side::Upper);

            // region は子のちょうど一方に含まれる。
            if lower_id.intersection(region).is_some() {
                if keep {
                    *upper_child = empty_leaf.clone();
                }
                Self::prune_path(lower_child, lower_id, region, keep, empty_leaf);
            } else {
                if keep {
                    *lower_child = empty_leaf.clone();
                }
                Self::prune_path(upper_child, upper_id, region, keep, empty_leaf);
            }

            *leaf_count = lower_child.leaf_count() + upper_child.leaf_count();
            *max_zoom = Node::<V>::fold_max_zoom(*level, lower_child, upper_child);
        }
    }
}
