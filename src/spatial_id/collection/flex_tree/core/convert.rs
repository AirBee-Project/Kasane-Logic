use alloc::vec::Vec;

use super::{node::Node, split_child_id};
use crate::{FlexId, FlexTreeCore, Side, SingleId};

pub struct LeavesIter<'a, V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue,
{
    pub stack: Vec<(&'a super::node::Node<V>, FlexId)>,
}

/// 葉ノードを参照のまま辿るイテレータである。
pub struct LeavesIterRef<'a, V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue,
{
    pub stack: Vec<(&'a super::node::Node<V>, FlexId)>,
}

/// 葉ノードの所有権を消費して辿るイテレータである。
pub struct LeavesIntoIter<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue,
{
    pub stack: Vec<(super::ptr::SharedNode<super::node::Node<V>>, FlexId)>,
}

impl<'a, V> Iterator for LeavesIter<'a, V>
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
                    let axis = Node::<V>::axis(*level);
                    let next_id = split_child_id(&current_id, axis, Side::Upper);
                    self.stack.push((upper_child.as_ref(), next_id));

                    let next_id = split_child_id(&current_id, axis, Side::Lower);
                    self.stack.push((lower_child.as_ref(), next_id));
                }
                Node::Leaf { value: Some(value) } => {
                    return Some((current_id, value.clone()));
                }
                Node::Leaf { value: None } => {
                    // Skip empty regions
                }
            }
        }

        None
    }
}

impl<V> Iterator for LeavesIntoIter<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue,
{
    type Item = (FlexId, V);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((node_ptr, current_id)) = self.stack.pop() {
            let node = match super::ptr::try_unwrap(node_ptr) {
                Ok(node) => node,
                Err(shared) => (*shared).clone(),
            };

            match node {
                Node::Branch {
                    level,
                    lower_child,
                    upper_child,
                    ..
                } => {
                    let axis = Node::<V>::axis(level);
                    let next_id = split_child_id(&current_id, axis, Side::Upper);
                    self.stack.push((upper_child, next_id));

                    let next_id = split_child_id(&current_id, axis, Side::Lower);
                    self.stack.push((lower_child, next_id));
                }
                Node::Leaf { value: Some(value) } => {
                    return Some((current_id, value));
                }
                Node::Leaf { value: None } => {
                    // Skip empty regions
                }
            }
        }

        None
    }
}

impl<'a, V> Iterator for LeavesIterRef<'a, V>
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
                    let axis = Node::<V>::axis(*level);
                    let next_id = split_child_id(&current_id, axis, Side::Upper);
                    self.stack.push((upper_child.as_ref(), next_id));

                    let next_id = split_child_id(&current_id, axis, Side::Lower);
                    self.stack.push((lower_child.as_ref(), next_id));
                }
                Node::Leaf { value: Some(value) } => {
                    return Some((current_id, value));
                }
                Node::Leaf { value: None } => {
                    // Skip empty regions
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
    pub fn single_ids(&self) -> impl Iterator<Item = SingleId> + '_ {
        self.iter()
            .flat_map(|(flex_id, _value)| flex_id.single_ids())
    }
}
