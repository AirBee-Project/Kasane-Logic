use crate::{Dimension, FlexId, IntoFlexIds};
use node::Node;
mod node;

#[derive(Debug, PartialEq, Clone)]
pub struct FlexTree {
    pub lower_root: Option<Box<Node>>,
    pub upper_root: Option<Box<Node>>,
}

impl FlexTree {
    /// 新しい空のツリーを作成する
    pub fn new() -> Self {
        Self {
            lower_root: None,
            upper_root: None,
        }
    }

    pub fn insert<T: IntoFlexIds>(&mut self, target: T) {
        for flex_id in target.into_flex_ids() {
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
                kd_node.insert(flex_id, 0, 0, 0);
            }
        }
    }

    pub fn output(&self) -> Vec<FlexId> {
        let mut results = Vec::new();

        if let Some(lower) = &self.lower_root {
            lower.collect_leaves(&mut results, FlexId::LOWER_MAX);
        }

        if let Some(upper) = &self.upper_root {
            upper.collect_leaves(&mut results, FlexId::UPPER_MAX);
        }

        results
    }
}
