use alloc::boxed::Box;

use crate::{FlexId, Side, spatial_id::collection::flex_tree::core::node::Dimension};

pub struct FlexTreeCore2<V> {
    upper_root: Node<V>,
    lower_root: Node<V>,
}

impl<V: Clone + PartialEq> Default for FlexTreeCore2<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: Clone + PartialEq> FlexTreeCore2<V> {
    /// 空の[FlexTreeCore2]を作成する
    pub fn new() -> Self {
        FlexTreeCore2 {
            upper_root: Node::Leaf(None),
            lower_root: Node::Leaf(None),
        }
    }

    /// [FlexId]と値を挿入する。
    /// 既に値がある場合には上書きされる。
    pub fn insert(&mut self, target: FlexId, value: V) {
        // 地球の北半球か南半球かを判定する（F=0 は上側に属する）
        let (root_node, root_flex_id) = if target.f_index().is_negative() {
            (&mut self.lower_root, FlexId::LOWER_MAX)
        } else {
            (&mut self.upper_root, FlexId::UPPER_MAX)
        };

        // 当該のNodeに挿入する
        root_node.insert(root_flex_id, target, value);
    }
}

pub enum Node<V> {
    Leaf(Option<V>),
    Branch {
        /// このBranchが分割している次元
        dimension: Dimension,
        /// Branch自身の領域の `dimension` のズームレベル。
        zoomlevel: u8,
        upper: Box<Node<V>>,
        lower: Box<Node<V>>,
    },
}

impl<V: Clone + PartialEq> Node<V> {
    pub fn insert(&mut self, this: FlexId, target: FlexId, value: V) {
        debug_assert!(this.contains(&target));

        match self {
            // 領域がちょうど一致したら、配下を丸ごと上書きする
            _ if this == target => *self = Node::Leaf(Some(value)),

            // 既に同じ値で埋まっているなら何もしない
            Node::Leaf(existing) if existing.as_ref() == Some(&value) => {}

            // target の方が細かいので、Leaf を分割して Branch にしてから挿入し直す
            Node::Leaf(existing) => {
                *self = Node::split_leaf(existing.take(), &this, &target);
                self.insert(this, target, value);
            }

            // 子の領域と target の交差を、それぞれの子へ挿入する
            Node::Branch {
                dimension,
                upper,
                lower,
                ..
            } => {
                for (side, child) in [(Side::Upper, upper), (Side::Lower, lower)] {
                    // Branch はより細かい target が来たときにしか作られないので、分割軸は最大ズーム未満
                    let child_id = this.split_on(*dimension, side).unwrap();
                    if let Some(child_target) = target.intersection(&child_id) {
                        child.insert(child_id, child_target, value.clone());
                    }
                }
                self.try_merge();
            }
        }
    }

    /// 値 `existing_leaf` で埋まった領域 `this` を、`target` より粗い最初の軸（F→X→Y→T）で二分した Branch を作る。
    fn split_leaf(existing_leaf: Option<V>, this: &FlexId, target: &FlexId) -> Self {
        let dimension = [Dimension::F, Dimension::X, Dimension::Y, Dimension::T]
            .into_iter()
            .find(|&axis| this.zoomlevel_on(axis) < target.zoomlevel_on(axis))
            .expect("target は this に真に包含されている");

        Node::Branch {
            dimension,
            zoomlevel: this.zoomlevel_on(dimension),
            upper: Box::new(Node::Leaf(existing_leaf.clone())),
            lower: Box::new(Node::Leaf(existing_leaf)),
        }
    }

    /// `upper_node`と`lower_node`が同じ`V`を持つなら自身を[Node::Leaf]にして`V`を入れる
    fn try_merge(&mut self) {
        if let Node::Branch { upper, lower, .. } = self
            && let (Node::Leaf(u), Node::Leaf(l)) = (upper.as_mut(), lower.as_mut())
            && u == l
        {
            *self = Node::Leaf(u.take());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leaf_count<V>(node: &Node<V>) -> usize {
        match node {
            Node::Leaf(Some(_)) => 1,
            Node::Leaf(None) => 0,
            Node::Branch { upper, lower, .. } => leaf_count(upper) + leaf_count(lower),
        }
    }

    #[test]
    fn f_zero_goes_to_upper_root() {
        let mut tree = FlexTreeCore2::new();
        tree.insert(FlexId::new(3, 0, 3, 0, 3, 0).unwrap(), 1u64);
        assert_eq!(leaf_count(&tree.upper_root), 1);
        assert_eq!(leaf_count(&tree.lower_root), 0);
    }

    #[test]
    fn insert_whole_root_becomes_single_leaf() {
        let mut tree = FlexTreeCore2::new();
        tree.insert(FlexId::UPPER_MAX, 7u64);
        assert!(matches!(tree.upper_root, Node::Leaf(Some(7))));
    }

    #[test]
    fn sibling_halves_with_same_value_try_merge() {
        let mut tree = FlexTreeCore2::new();
        tree.insert(FlexId::new(1, 0, 0, 0, 0, 0).unwrap(), 5u64);
        tree.insert(FlexId::new(1, 1, 0, 0, 0, 0).unwrap(), 5u64);
        assert!(matches!(tree.upper_root, Node::Leaf(Some(5))));
    }

    #[test]
    fn overwrite_inside_filled_leaf_splits() {
        let mut tree = FlexTreeCore2::new();
        tree.insert(FlexId::UPPER_MAX, 1u64);
        tree.insert(FlexId::new(2, 1, 2, 3, 2, 0).unwrap(), 2u64);
        // 1 の領域が F,X,Y に沿って切り分けられ、2 の葉が1つ入る
        assert!(leaf_count(&tree.upper_root) > 2);

        // 同じ場所を元の値で上書きすると、全体が1つの葉に戻る
        tree.insert(FlexId::new(2, 1, 2, 3, 2, 0).unwrap(), 1u64);
        assert!(matches!(tree.upper_root, Node::Leaf(Some(1))));
    }
}
