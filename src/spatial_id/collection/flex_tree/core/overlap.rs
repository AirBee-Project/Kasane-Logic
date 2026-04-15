use crate::{
    Dimension, FlexId, FlexTree, Side, spatial_id::collection::flex_tree::core::node::Node,
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

impl<V> FlexTree<V>
where
    V: Clone + PartialEq,
{
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
}
