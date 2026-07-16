use alloc::vec::Vec;

use super::ptr::SharedNode;
use super::{FlexTreeCore, split_child_id};
use crate::spatial_id::collection::flex_tree::core::node::{Node, OverlappingChildren};
use crate::{FlexId, Side};

/// 重なり合う領域のみを遅延評価で探索するイテレータ
pub struct OverlapIter<'a, V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue,
{
    target: FlexId,
    stack: Vec<(&'a Node<V>, FlexId)>,
}

/// 重なり合う領域のみを参照付きで遅延評価で探索するイテレータ
pub struct OverlapIterRef<'a, V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue,
{
    target: FlexId,
    stack: Vec<(&'a Node<V>, FlexId)>,
}

/// Branch を降りる際、target と交差しうる子だけを積む。
///
/// 交差しない子は積む前に捨てるため、点クエリでは葉まで一直線に降りる。
/// これにより、積んだ後に [`FlexId::intersection`] で捨て直す必要がなくなる。
fn push_overlapping_children<'a, V>(
    stack: &mut Vec<(&'a Node<V>, FlexId)>,
    target: &FlexId,
    level: u8,
    lower_child: &'a SharedNode<Node<V>>,
    upper_child: &'a SharedNode<Node<V>>,
    current_id: &FlexId,
) where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue,
{
    let axis = Node::<V>::axis(level);
    let mut push = |side: Side, child: &'a SharedNode<Node<V>>| {
        stack.push((child.as_ref(), split_child_id(current_id, axis, side)));
    };

    match Node::<V>::overlapping_children(target, level) {
        OverlappingChildren::Both => {
            push(Side::Upper, upper_child);
            push(Side::Lower, lower_child);
        }
        OverlappingChildren::Only(Side::Lower) => push(Side::Lower, lower_child),
        OverlappingChildren::Only(Side::Upper) => push(Side::Upper, upper_child),
    }
}

impl<'a, V> Iterator for OverlapIter<'a, V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue,
{
    type Item = (FlexId, V);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((node, current_id)) = self.stack.pop() {
            match node {
                Node::Branch {
                    level,
                    lower_child,
                    upper_child,
                    ..
                } => {
                    push_overlapping_children(
                        &mut self.stack,
                        &self.target,
                        *level,
                        lower_child,
                        upper_child,
                        &current_id,
                    );
                }
                // 交差する子しか積んでいないので、辿り着いた葉は必ず target と交差する。
                Node::Leaf { value: Some(value) } => {
                    return Some((current_id, value.clone()));
                }
                Node::Leaf { value: None } => {
                    // Skip
                }
            }
        }
        None
    }
}

impl<'a, V> Iterator for OverlapIterRef<'a, V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue,
{
    type Item = (FlexId, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((node, current_id)) = self.stack.pop() {
            match node {
                Node::Branch {
                    level,
                    lower_child,
                    upper_child,
                    ..
                } => {
                    push_overlapping_children(
                        &mut self.stack,
                        &self.target,
                        *level,
                        lower_child,
                        upper_child,
                        &current_id,
                    );
                }
                Node::Leaf { value: Some(value) } => {
                    return Some((current_id, value));
                }
                Node::Leaf { value: None } => {
                    // Skip
                }
            }
        }
        None
    }
}

impl<V> FlexTreeCore<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue,
{
    pub fn overlap(&self, target: FlexId) -> impl Iterator<Item = (FlexId, V)> + '_ {
        OverlapIter {
            stack: self.overlap_root_stack(&target),
            target,
        }
    }

    pub fn overlap_ref(&self, target: FlexId) -> impl Iterator<Item = (FlexId, &V)> + '_ {
        OverlapIterRef {
            stack: self.overlap_root_stack(&target),
            target,
        }
    }

    /// 走査開始点として、target と交差しうるルートだけを ID 付きで収集する。
    ///
    /// F はズーム0で `0`（上半分）と `-1`（下半分）の2セルしか持たないため、
    /// [`FlexId`] は必ずどちらか一方に収まる。[`insert_flex_id`](Self::insert_flex_id)
    /// と同じく符号でルートを選べば、もう一方の半空間は走査せずに済む。
    fn overlap_root_stack(&self, target: &FlexId) -> Vec<(&Node<V>, FlexId)> {
        let (root, root_id) = if target.f_index().is_negative() {
            (&self.lower_root, FlexId::LOWER_MAX)
        } else {
            (&self.upper_root, FlexId::UPPER_MAX)
        };

        if SharedNode::ptr_eq(root, &self.empty_leaf) {
            Vec::new()
        } else {
            alloc::vec![(root.as_ref(), root_id)]
        }
    }

    pub fn overlap_remove(&mut self, target: &FlexId) -> impl Iterator<Item = (FlexId, V)> {
        let mut removed_items = Vec::new();

        // 走査と同じく、target が属する側のルートだけを掘る。
        let (root, root_id) = if target.f_index().is_negative() {
            (&mut self.lower_root, FlexId::LOWER_MAX)
        } else {
            (&mut self.upper_root, FlexId::UPPER_MAX)
        };
        Self::prune_node_mut(root, target, root_id, &mut removed_items, &self.empty_leaf);

        removed_items.into_iter()
    }

    /// `current_id` が指す部分木から target と重なる葉を取り除く。
    ///
    /// 呼び出し側は `current_id` が target と交差することを保証する
    /// （[`overlap_remove`](Self::overlap_remove) が符号でルートを選び、以降は
    /// [`Node::overlapping_children`] が交差する子しか降りないため）。
    fn prune_node_mut(
        node: &mut SharedNode<Node<V>>,
        target: &FlexId,
        current_id: FlexId,
        removed: &mut Vec<(FlexId, V)>,
        empty_leaf: &SharedNode<Node<V>>,
    ) {
        if let Node::Leaf { value: None } = **node {
            return;
        }

        if let Node::Leaf { value: Some(ref v) } = **node {
            removed.push((current_id, v.clone()));
            *node = empty_leaf.clone();
            return;
        }

        {
            let mut_node = SharedNode::make_mut(node);
            let Node::Branch {
                level,
                lower_child,
                upper_child,
                leaf_count,
                max_zoom,
            } = mut_node
            else {
                unreachable!("葉は上で処理済み")
            };

            let axis = Node::<V>::axis(*level);
            match Node::<V>::overlapping_children(target, *level) {
                OverlappingChildren::Both => {
                    let upper_id = split_child_id(&current_id, axis, Side::Upper);
                    Self::prune_node_mut(upper_child, target, upper_id, removed, empty_leaf);

                    let lower_id = split_child_id(&current_id, axis, Side::Lower);
                    Self::prune_node_mut(lower_child, target, lower_id, removed, empty_leaf);
                }
                OverlappingChildren::Only(side) => {
                    let child = match side {
                        Side::Lower => &mut *lower_child,
                        Side::Upper => &mut *upper_child,
                    };
                    let child_id = split_child_id(&current_id, axis, side);
                    Self::prune_node_mut(child, target, child_id, removed, empty_leaf);
                }
            }

            *leaf_count = (lower_child.leaf_count() + upper_child.leaf_count()) as u32;
            *max_zoom = Node::<V>::fold_max_zoom(*level, lower_child, upper_child);
        }

        if node.leaf_count() == 0 {
            *node = empty_leaf.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::spatial_id::collection::flex_tree::core::FlexTreeCore;
    use crate::{FlexId, RangeId, SingleId};
    use alloc::collections::BTreeSet;
    use alloc::vec::Vec;
    use proptest::prelude::*;

    /// 枝刈りを一切せず、全葉を走査して交差するものだけを残す素朴な実装。
    ///
    /// [`FlexTreeCore::overlap_ref`] はビット演算で交差しない枝を落とすが、
    /// その枝刈りが「交差する葉を取りこぼさない」ことは木の形に依存して壊れうる。
    /// ここでは木の形に依存しない基準として、この総当たりと突き合わせる。
    fn brute_force_overlap<V: super::super::ptr::SafeValue>(
        core: &FlexTreeCore<V>,
        target: &FlexId,
    ) -> Vec<FlexId> {
        let mut found: Vec<FlexId> = core
            .iter_ref()
            .filter(|(leaf_id, _)| leaf_id.intersection(target).is_some())
            .map(|(leaf_id, _)| leaf_id)
            .collect();
        found.sort();
        found
    }

    fn overlap_ids<V: super::super::ptr::SafeValue>(
        core: &FlexTreeCore<V>,
        target: &FlexId,
    ) -> Vec<FlexId> {
        let mut found: Vec<FlexId> = core.overlap_ref(target.clone()).map(|(id, _)| id).collect();
        found.sort();
        found
    }

    prop_compose! {
        fn arb_single_id()(
            z in 0u8..6,
            f in -8i32..8,
            x in 0u32..32,
            y in 0u32..32,
        ) -> SingleId {
            let max_xy = (1u32 << z) - 1;
            let f_max = if z == 0 { 0 } else { (1i32 << z) - 1 };
            let f_clamped = f.clamp(-f_max - 1, f_max);
            SingleId::new(z, f_clamped, x.min(max_xy), y.min(max_xy)).unwrap()
        }
    }

    proptest! {
        /// 点クエリ（SingleId）の結果が総当たりと一致することを検証する。
        #[test]
        fn overlap_matches_brute_force_for_point_query(
            inserts in prop::collection::vec(arb_single_id(), 1..12),
            queries in prop::collection::vec(arb_single_id(), 1..8),
        ) {
            let mut core: FlexTreeCore<u32> = FlexTreeCore::new();
            for (i, id) in inserts.iter().enumerate() {
                core.insert(id.clone(), i as u32);
            }

            for query in &queries {
                for target in query.clone().into_iter() {
                    prop_assert_eq!(
                        overlap_ids(&core, &target),
                        brute_force_overlap(&core, &target),
                        "target={:?}", target
                    );
                }
            }
        }
    }

    prop_compose! {
        /// 異方セル（軸ごとにズームが異なる FlexId）を含む RangeId を生成する。
        fn arb_range_id()(
            z in 1u8..5,
            f0 in -6i32..6,
            df in 0i32..5,
            x0 in 0u32..12,
            dx in 0u32..5,
            y0 in 0u32..12,
            dy in 0u32..5,
        ) -> RangeId {
            let max_xy = (1u32 << z) - 1;
            let f_max = (1i32 << z) - 1;
            let fa = f0.clamp(-f_max - 1, f_max);
            let fb = (fa + df).clamp(-f_max - 1, f_max);
            let xa = x0.min(max_xy);
            let xb = (xa + dx).min(max_xy);
            let ya = y0.min(max_xy);
            let yb = (ya + dy).min(max_xy);
            RangeId::new(z, [fa, fb], [xa, xb], [ya, yb]).unwrap()
        }
    }

    proptest! {
        /// 範囲クエリ、および RangeId 挿入で生じる異方セルでも総当たりと一致することを検証する。
        #[test]
        fn overlap_matches_brute_force_for_range_query(
            inserts in prop::collection::vec(arb_range_id(), 1..6),
            queries in prop::collection::vec(arb_range_id(), 1..4),
        ) {
            let mut core: FlexTreeCore<u32> = FlexTreeCore::new();
            for (i, id) in inserts.iter().enumerate() {
                core.insert(id.clone(), i as u32);
            }

            for query in &queries {
                for target in query.clone().into_iter() {
                    prop_assert_eq!(
                        overlap_ids(&core, &target),
                        brute_force_overlap(&core, &target),
                        "target={:?}", target
                    );
                }
            }
        }
    }

    /// ツリーの内容を、ズーム `z` の [`SingleId`] 集合へ展開する。
    ///
    /// 葉の [`FlexId`] は異方セルでありうるため、木の形に依存しない粒度へ均して比較する。
    fn single_ids_at<V: super::super::ptr::SafeValue>(
        core: &FlexTreeCore<V>,
        z: u8,
    ) -> BTreeSet<SingleId> {
        core.iter_ref()
            .flat_map(|(flex_id, _)| flex_id_cells(&flex_id, z))
            .collect()
    }

    /// 単一の [`FlexId`] を、ズーム `z` の [`SingleId`] 集合へ展開する。
    fn flex_id_cells(flex_id: &FlexId, z: u8) -> BTreeSet<SingleId> {
        let range = RangeId::from(flex_id);
        let normalized = if range.z() == z {
            range
        } else {
            range.spatial_children_at_zoom(z).unwrap()
        };
        normalized.single_ids().collect()
    }

    proptest! {
        /// `remove` が「target のセルだけを」過不足なく取り除くことを、
        /// SingleId 粒度のモデルと突き合わせて検証する。
        ///
        /// 既存の remove テストは件数（count）しか見ておらず、
        /// 「どのセルが残ったか」は固定ケースでしか押さえられていない。
        #[test]
        fn remove_removes_exactly_the_target_cells(
            inserts in prop::collection::vec(arb_single_id(), 1..10),
            target in arb_range_id(),
        ) {
            let z = inserts
                .iter()
                .map(|id| id.z())
                .chain(core::iter::once(target.z()))
                .max()
                .unwrap();

            let mut tree: FlexTreeCore<u32> = FlexTreeCore::new();
            for (i, id) in inserts.iter().enumerate() {
                tree.insert(id.clone(), i as u32);
            }

            let before = single_ids_at(&tree, z);
            let target_cells: BTreeSet<SingleId> = target
                .clone()
                .into_iter()
                .flat_map(|t| flex_id_cells(&t, z))
                .collect();

            let removed: Vec<(FlexId, u32)> = tree.remove(target.clone()).collect();
            let removed_cells: BTreeSet<SingleId> = removed
                .iter()
                .flat_map(|(id, _)| flex_id_cells(id, z))
                .collect();
            let after = single_ids_at(&tree, z);

            // 残るのは「元 - target」。
            prop_assert_eq!(
                after,
                before.difference(&target_cells).cloned().collect::<BTreeSet<_>>()
            );
            // 返るのは「元 ∩ target」。
            prop_assert_eq!(
                removed_cells,
                before.intersection(&target_cells).cloned().collect::<BTreeSet<_>>()
            );
        }
    }
}
