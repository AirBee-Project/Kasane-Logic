use crate::{
    Dimension, FlexId, IntoFlexIds, IterFlexIds,
    spatial_id::collection::flex_tree::core::convert::LeavesIter,
};
use node::Node;
mod convert;
mod node;
mod overlap;

#[derive(PartialEq, Clone)]
pub struct FlexTree<V>
where
    V: PartialEq + Clone,
{
    pub lower_root: Option<Box<Node<V>>>,
    pub upper_root: Option<Box<Node<V>>>,
}

impl<V> FlexTree<V>
where
    V: PartialEq + Clone,
{
    /// 新しい空のツリーを作成する
    pub fn new() -> Self {
        Self {
            lower_root: None,
            upper_root: None,
        }
    }

    pub fn insert<S>(&mut self, target: S, value: V)
    where
        S: IterFlexIds,
    {
        for flex_id in target.iter_flex_ids() {
            let root = match flex_id.f_index().is_negative() {
                true => &mut self.lower_root,
                false => &mut self.upper_root,
            };

            if root.is_none() {
                *root = Some(Box::new(Node::Branch {
                    axis: Dimension::F,
                    lower_child: None,
                    upper_child: None,
                }));
            }

            if let Some(kd_node) = root {
                kd_node.insert(flex_id, value.clone(), 0, 0, 0);
            }
        }
    }

    pub fn get<'a, S>(&'a self, target: &'a S) -> impl Iterator<Item = (FlexId, V)> + 'a
    where
        S: IterFlexIds,
    {
        target
            .iter_flex_ids()
            .flat_map(move |item| self.overlap(item))
    }

    pub fn iter(&self) -> impl Iterator<Item = (FlexId, V)> + '_ {
        let mut stack = Vec::new();

        if let Some(upper) = &self.upper_root {
            stack.push((upper.as_ref(), FlexId::UPPER_MAX));
        }

        if let Some(lower) = &self.lower_root {
            stack.push((lower.as_ref(), FlexId::LOWER_MAX));
        }

        LeavesIter { stack }
    }
}
