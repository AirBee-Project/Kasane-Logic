use alloc::vec::Vec;

use super::node::Node;
use super::ptr::{SafeValue, SharedNode};
use super::split_child_id;
use crate::{FlexId, FlexTreeCore, Side, SingleId};

/// 葉ノードを参照のまま辿るイテレータ。所有権を持つ [`FlexTreeCore::iter`] は
/// これを `map(clone)` して構築するため、走査ロジックはここ 1 か所に集約されている。
pub struct LeavesIterRef<'a, V>
where
    V: SafeValue,
{
    pub stack: Vec<(&'a Node<V>, FlexId)>,
}

/// 葉ノードの所有権を消費して辿るイテレータ。`Rc`/`Arc` が一意なら値をムーブで取り出す。
pub struct LeavesIntoIter<V>
where
    V: SafeValue,
{
    pub stack: Vec<(SharedNode<Node<V>>, FlexId)>,
}

/// Branch を降りるとき、下・上の子を ID 付きでスタックへ積む（葉走査の共通処理）。
#[inline]
fn push_children<T>(
    stack: &mut Vec<(T, FlexId)>,
    level: u8,
    current_id: &FlexId,
    lower: T,
    upper: T,
) {
    let axis = Node::<()>::axis(level);
    stack.push((upper, split_child_id(current_id, axis, Side::Upper)));
    stack.push((lower, split_child_id(current_id, axis, Side::Lower)));
}

impl<'a, V> Iterator for LeavesIterRef<'a, V>
where
    V: SafeValue,
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
                } => push_children(
                    &mut self.stack,
                    *level,
                    &current_id,
                    lower_child.as_ref(),
                    upper_child.as_ref(),
                ),
                Node::Leaf { value: Some(value) } => return Some((current_id, value)),
                Node::Leaf { value: None } => {}
            }
        }
        None
    }
}

impl<V> Iterator for LeavesIntoIter<V>
where
    V: SafeValue,
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
                } => push_children(
                    &mut self.stack,
                    level,
                    &current_id,
                    lower_child,
                    upper_child,
                ),
                Node::Leaf { value: Some(value) } => return Some((current_id, value)),
                Node::Leaf { value: None } => {}
            }
        }
        None
    }
}

impl<V> FlexTreeCore<V>
where
    V: SafeValue,
{
    pub fn single_ids(&self) -> impl Iterator<Item = SingleId> + '_ {
        self.iter()
            .flat_map(|(flex_id, _value)| flex_id.single_ids())
    }
}
