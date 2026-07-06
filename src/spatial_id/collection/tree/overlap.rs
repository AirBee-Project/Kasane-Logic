#![allow(dead_code)]
use alloc::vec::Vec;

use super::ptr::SharedNode;
use super::{FlexTreeCore, split_child_id};
use crate::{Dimension, FlexId, Side, spatial_id::collection::tree::node::Node};

/// 重なり合う領域のみを遅延評価で探索するイテレータ
pub struct OverlapIter<'a, V>
where
    V: crate::spatial_id::collection::tree::ptr::SafeValue,
{
    target: FlexId,
    stack: Vec<(&'a Node<V>, FlexId)>,
}

/// 重なり合う領域のみを参照付きで遅延評価で探索するイテレータ
pub struct OverlapIterRef<'a, V>
where
    V: crate::spatial_id::collection::tree::ptr::SafeValue,
{
    target: FlexId,
    stack: Vec<(&'a Node<V>, FlexId)>,
}

impl<'a, V> OverlapIter<'a, V>
where
    V: crate::spatial_id::collection::tree::ptr::SafeValue,
{
    fn push_branch_children(
        &mut self,
        axis: Dimension,
        lower_child: &'a SharedNode<Node<V>>,
        upper_child: &'a SharedNode<Node<V>>,
        current_id: &FlexId,
    ) {
        self.stack.push((
            upper_child.as_ref(),
            split_child_id(current_id, axis, Side::Upper),
        ));
        self.stack.push((
            lower_child.as_ref(),
            split_child_id(current_id, axis, Side::Lower),
        ));
    }
}

impl<'a, V> Iterator for OverlapIter<'a, V>
where
    V: crate::spatial_id::collection::tree::ptr::SafeValue,
{
    type Item = (FlexId, V);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((node, current_id)) = self.stack.pop() {
            if current_id.intersection(&self.target).is_none() {
                continue;
            }

            match node {
                Node::Branch {
                    level,
                    lower_child,
                    upper_child,
                    ..
                } => {
                    let axis = Node::<V>::axis(*level);
                    self.push_branch_children(axis, lower_child, upper_child, &current_id);
                }
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
    V: crate::spatial_id::collection::tree::ptr::SafeValue,
{
    type Item = (FlexId, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((node, current_id)) = self.stack.pop() {
            if current_id.intersection(&self.target).is_none() {
                continue;
            }

            match node {
                Node::Branch {
                    level,
                    lower_child,
                    upper_child,
                    ..
                } => {
                    let axis = Node::<V>::axis(*level);
                    self.stack.push((
                        upper_child.as_ref(),
                        split_child_id(&current_id, axis, Side::Upper),
                    ));
                    self.stack.push((
                        lower_child.as_ref(),
                        split_child_id(&current_id, axis, Side::Lower),
                    ));
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
    V: crate::spatial_id::collection::tree::ptr::SafeValue,
{
    pub fn overlap(&self, target: FlexId) -> impl Iterator<Item = (FlexId, V)> + '_ {
        OverlapIter {
            target,
            stack: self.root_node_stack(),
        }
    }

    pub fn overlap_ref(&self, target: FlexId) -> impl Iterator<Item = (FlexId, &V)> + '_ {
        OverlapIterRef {
            target,
            stack: self.root_node_stack(),
        }
    }

    pub fn overlap_remove(&mut self, target: &FlexId) -> impl Iterator<Item = (FlexId, V)> {
        let mut removed_items = Vec::new();
        Self::prune_node_mut(
            &mut self.lower_root,
            target,
            FlexId::LOWER_MAX,
            &mut removed_items,
            &self.empty_leaf,
        );
        Self::prune_node_mut(
            &mut self.upper_root,
            target,
            FlexId::UPPER_MAX,
            &mut removed_items,
            &self.empty_leaf,
        );
        removed_items.into_iter()
    }

    fn prune_node_mut(
        node: &mut SharedNode<Node<V>>,
        target: &FlexId,
        current_id: FlexId,
        removed: &mut Vec<(FlexId, V)>,
        empty_leaf: &SharedNode<Node<V>>,
    ) {
        if current_id.intersection(target).is_none() {
            return;
        }

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
            if let Node::Branch {
                level,
                lower_child,
                upper_child,
                leaf_count,
                max_zoom,
            } = mut_node
            {
                let axis = Node::<V>::axis(*level);

                let upper_id = split_child_id(&current_id, axis, Side::Upper);
                Self::prune_node_mut(upper_child, target, upper_id, removed, empty_leaf);

                let lower_id = split_child_id(&current_id, axis, Side::Lower);
                Self::prune_node_mut(lower_child, target, lower_id, removed, empty_leaf);

                *leaf_count = lower_child.leaf_count() + upper_child.leaf_count();
                *max_zoom = Node::<V>::fold_max_zoom(*level, lower_child, upper_child);
            } else {
                unreachable!()
            }
        }

        if node.leaf_count() == 0 {
            *node = empty_leaf.clone();
        }
    }
}
