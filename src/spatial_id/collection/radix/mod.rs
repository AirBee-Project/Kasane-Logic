use std::vec;

use crate::{
    FlexId,
    spatial_id::collection::radix::node::{Axis, KDNode},
};

pub mod convert;
pub mod node;
#[derive(Debug, PartialEq, Clone)]
pub struct KDTree {
    pub lower_root: Option<Box<KDNode>>,
    pub upper_root: Option<Box<KDNode>>,
}

impl KDTree {
    /// 新しい空のツリーを作成する
    pub fn new() -> Self {
        Self {
            lower_root: None,
            upper_root: None,
        }
    }

    pub fn insert(&mut self, target: FlexId) {
        //lowerとupperどちらに所属するのかを判定する
        let root = match target.f_index().is_negative() {
            true => &mut self.lower_root,
            false => &mut self.upper_root,
        };

        if root.is_none() {
            *root = Some(Box::new(KDNode::Leaf));
        }

        if let Some(kd_node) = root {
            kd_node.insert(target, 0, 0, 0);
        }
    }

    /// ツリー内に存在するすべての拡張空間ID（重なりが排除されたSet）を出力します。
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
