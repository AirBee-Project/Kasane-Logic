use crate::{
    Dimension, FlexId, FlexTreeCore, Side, spatial_id::collection::flex_tree::core::node::Node,
};

/// 重なり合う領域のみを遅延評価で探索するイテレータ
pub struct OverlapIter<'a, V>
where
    V: Clone + PartialEq,
{
    target: FlexId,
    stack: Vec<(&'a Node<V>, FlexId)>,
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
                    axis,
                    lower_child,
                    upper_child,
                } => {
                    if let Some(child) = upper_child {
                        let next_id = match axis {
                            Dimension::F => current_id.f_split(Side::Upper).unwrap(),
                            Dimension::X => current_id.x_split(Side::Upper).unwrap(),
                            Dimension::Y => current_id.y_split(Side::Upper).unwrap(),
                        };
                        self.stack.push((child.as_ref(), next_id));
                    }

                    if let Some(child) = lower_child {
                        let next_id = match axis {
                            Dimension::F => current_id.f_split(Side::Lower).unwrap(),
                            Dimension::X => current_id.x_split(Side::Lower).unwrap(),
                            Dimension::Y => current_id.y_split(Side::Lower).unwrap(),
                        };
                        self.stack.push((child.as_ref(), next_id));
                    }
                }
                Node::Leaf { value } => {
                    return Some((current_id, value.clone()));
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
    ///Targetと重なっている[FlexId]とValueを全て取得する関数
    pub fn overlap(&self, target: FlexId) -> impl Iterator<Item = (FlexId, V)> + '_ {
        let mut stack = Vec::new();

        if let Some(upper) = &self.upper_root {
            stack.push((upper.as_ref(), FlexId::UPPER_MAX));
        }
        if let Some(lower) = &self.lower_root {
            stack.push((lower.as_ref(), FlexId::LOWER_MAX));
        }

        OverlapIter {
            target: target.clone(),
            stack,
        }
    }

    ///Targetと重なっている[FlexId]とValueを全て削除し、取得する関数
    pub fn overlap_remove(&mut self, target: &FlexId) -> impl Iterator<Item = (FlexId, V)> {
        // 削除された要素を一時的に集めるためのベクタ
        let mut removed_items = Vec::new();

        Self::prune_node(
            &mut self.lower_root,
            target,
            FlexId::LOWER_MAX,
            &mut removed_items,
        );
        Self::prune_node(
            &mut self.upper_root,
            target,
            FlexId::UPPER_MAX,
            &mut removed_items,
        );
        removed_items.into_iter()
    }

    fn prune_node(
        node_opt: &mut Option<Box<Node<V>>>,
        target: &FlexId,
        current_id: FlexId,
        removed: &mut Vec<(FlexId, V)>,
    ) {
        if current_id.intersection(target).is_none() {
            return;
        }
        let mut node = match node_opt.take() {
            Some(n) => n,
            None => return,
        };
        let keep_node = match node.as_mut() {
            Node::Branch {
                axis,
                lower_child,
                upper_child,
            } => {
                if lower_child.is_some() {
                    let next_id = match axis {
                        Dimension::F => current_id.f_split(Side::Lower).unwrap(),
                        Dimension::X => current_id.x_split(Side::Lower).unwrap(),
                        Dimension::Y => current_id.y_split(Side::Lower).unwrap(),
                    };
                    Self::prune_node(lower_child, target, next_id, removed);
                }

                if upper_child.is_some() {
                    let next_id = match axis {
                        Dimension::F => current_id.f_split(Side::Upper).unwrap(),
                        Dimension::X => current_id.x_split(Side::Upper).unwrap(),
                        Dimension::Y => current_id.y_split(Side::Upper).unwrap(),
                    };
                    Self::prune_node(upper_child, target, next_id, removed);
                }

                lower_child.is_some() || upper_child.is_some()
            }
            Node::Leaf { .. } => false,
        };

        if keep_node {
            *node_opt = Some(node);
        } else {
            if let Node::Leaf { value } = *node {
                removed.push((current_id, value));
            }
        }
    }
}
