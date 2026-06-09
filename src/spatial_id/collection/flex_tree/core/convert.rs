#[allow(unused_imports)]
use alloc::boxed::Box;
#[allow(unused_imports)]
use alloc::rc::Rc;
#[allow(unused_imports)]
use alloc::string::{String, ToString};
#[allow(unused_imports)]
use alloc::vec::Vec;

use crate::spatial_id::collection::flex_tree::core::split_child_id;
use crate::{
    FlexId, FlexTreeCore, IntoFlexIds, IntoSingleIds, IterFlexIds, IterSingleIds, Side, SingleId,
    spatial_id::collection::flex_tree::core::node::Node,
};

pub struct LeavesIter<'a, V>
where
    V: Clone + PartialEq,
{
    pub stack: Vec<(&'a super::node::Node<V>, FlexId)>,
}

/// 葉ノードを参照のまま辿るイテレータである。
pub struct LeavesIterRef<'a, V>
where
    V: PartialEq + Clone,
{
    pub stack: Vec<(&'a super::node::Node<V>, FlexId)>,
}

impl<'a, V> Iterator for LeavesIter<'a, V>
where
    V: PartialEq + Clone,
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

impl<'a, V> Iterator for LeavesIterRef<'a, V>
where
    V: PartialEq + Clone,
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

impl<V> IntoFlexIds for FlexTreeCore<V>
where
    V: PartialEq + Clone,
{
    type IntoIter = alloc::vec::IntoIter<FlexId>;

    fn into_flex_ids(self) -> Self::IntoIter {
        self.iter()
            .map(|(flex_id, _value)| flex_id)
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl<V> IterFlexIds for FlexTreeCore<V>
where
    V: PartialEq + Clone,
{
    type Iter<'a>
        = Box<dyn Iterator<Item = FlexId> + 'a>
    where
        Self: 'a;

    fn iter_flex_ids(&self) -> Self::Iter<'_> {
        Box::new(self.iter().map(|(flex_id, _value)| flex_id))
    }
}

impl<V> IntoSingleIds for FlexTreeCore<V>
where
    V: PartialEq + Clone,
{
    type IntoIter = alloc::vec::IntoIter<SingleId>;

    fn into_single_ids(self) -> Self::IntoIter {
        self.iter()
            .flat_map(|(flex_id, _value)| flex_id.into_single_ids())
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl<V> IterSingleIds for FlexTreeCore<V>
where
    V: PartialEq + Clone,
{
    type Iter<'a>
        = Box<dyn Iterator<Item = SingleId> + 'a>
    where
        Self: 'a;

    fn iter_single_ids(&self) -> Self::Iter<'_> {
        Box::new(
            self.iter()
                .flat_map(|(flex_id, _value)| flex_id.iter_single_ids().collect::<Vec<_>>()),
        )
    }
}
