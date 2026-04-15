use crate::{Dimension, FlexId, Side, spatial_id::collection::flex_tree::core::node::Node};

pub struct LeavesIter<'a, V>
where
    V: Clone + PartialEq,
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
