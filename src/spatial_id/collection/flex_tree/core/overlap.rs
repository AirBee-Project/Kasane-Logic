use crate::{
    Dimension, FlexId, FlexTreeCore, Side,
    spatial_id::collection::flex_tree::core::{node::Node, split_child_id},
};
use std::rc::Rc;

/// 重なり合う領域のみを遅延評価で探索するイテレータ
pub struct OverlapIter<'a, V>
where
    V: Clone + PartialEq,
{
    target: FlexId,
    stack: Vec<(&'a Node<V>, FlexId)>,
}

/// 重なり合う領域のみを参照付きで遅延評価で探索するイテレータ
pub struct OverlapIterRef<'a, V>
where
    V: Clone + PartialEq,
{
    target: FlexId,
    stack: Vec<(&'a Node<V>, FlexId)>,
}

impl<'a, V> OverlapIter<'a, V>
where
    V: PartialEq + Clone,
{
    fn push_branch_children(
        &mut self,
        axis: Dimension,
        lower_child: &'a Rc<Node<V>>,
        upper_child: &'a Rc<Node<V>>,
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
    V: PartialEq + Clone,
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
    V: PartialEq + Clone,
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
    V: Clone + PartialEq,
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
        self.lower_root = Self::prune_node(
            &self.lower_root,
            target,
            FlexId::LOWER_MAX,
            &mut removed_items,
            &self.empty_leaf,
        );
        self.upper_root = Self::prune_node(
            &self.upper_root,
            target,
            FlexId::UPPER_MAX,
            &mut removed_items,
            &self.empty_leaf,
        );
        removed_items.into_iter()
    }

    fn prune_node(
        node: &Rc<Node<V>>,
        target: &FlexId,
        current_id: FlexId,
        removed: &mut Vec<(FlexId, V)>,
        empty_leaf: &Rc<Node<V>>,
    ) -> Rc<Node<V>> {
        if current_id.intersection(target).is_none() {
            return node.clone();
        }

        match &**node {
            Node::Branch {
                level,
                lower_child,
                upper_child,
                ..
            } => {
                let axis = Node::<V>::axis(*level);
                let upper_id = split_child_id(&current_id, axis, Side::Upper);
                let new_upper =
                    Self::prune_node(upper_child, target, upper_id, removed, empty_leaf);

                let lower_id = split_child_id(&current_id, axis, Side::Lower);
                let new_lower =
                    Self::prune_node(lower_child, target, lower_id, removed, empty_leaf);

                if Rc::ptr_eq(&new_lower, lower_child) && Rc::ptr_eq(&new_upper, upper_child) {
                    return node.clone();
                }

                if new_lower.leaf_count() == 0 && new_upper.leaf_count() == 0 {
                    return empty_leaf.clone();
                }

                Rc::new(Node::Branch {
                    level: *level,
                    leaf_count: new_lower.leaf_count() + new_upper.leaf_count(),
                    lower_child: new_lower,
                    upper_child: new_upper,
                })
            }
            Node::Leaf { value: Some(v) } => {
                removed.push((current_id, v.clone()));
                empty_leaf.clone()
            }
            Node::Leaf { value: None } => node.clone(),
        }
    }
}
