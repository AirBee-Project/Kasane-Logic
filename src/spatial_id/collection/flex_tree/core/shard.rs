use super::{FlexTreeCore, node::Node, ptr::SharedNode, split_child_id};
use crate::{FlexId, Side};

impl<V> FlexTreeCore<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue,
{
    /// この[`FlexTreeCore`]をシャード分割すべきかを判定する。保持する[FlexId]数が `max_flex_id_count` を超えていれば `true`を返す。[FlexId]の個数はキャッシュされているため高速に動作する。
    pub fn should_split_shard(&self, max_flex_id_count: usize) -> bool {
        self.count() > max_flex_id_count
    }

    /// このシャード（[`shard`](Self::shard) 領域）を、現在のrootの軸で2分割し、切り取った部分木を `((下のシャード領域, 下の実体), (上のシャード領域, 上の実体))` で返す。
    /// シャード領域が未設定なら `None`を返す。
    pub(crate) fn split_shard(&self) -> Option<((FlexId, Self), (FlexId, Self))> {
        let region = self.shard()?.clone();
        let level = region.f_zoomlevel() + region.x_zoomlevel() + region.y_zoomlevel();
        let axis = Node::<V>::axis(level);
        let lower = split_child_id(&region, axis, Side::Lower);
        let upper = split_child_id(&region, axis, Side::Upper);
        Some((
            (lower.clone(), self.extract_region(lower)),
            (upper.clone(), self.extract_region(upper)),
        ))
    }

    pub(crate) fn extract_region(&self, region: FlexId) -> Self {
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

            *leaf_count = (lower_child.leaf_count() + upper_child.leaf_count()) as u32;
            *max_zoom = Node::<V>::fold_max_zoom(*level, lower_child, upper_child);
        }
    }
}
