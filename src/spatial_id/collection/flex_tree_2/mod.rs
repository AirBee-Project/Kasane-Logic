use crate::FlexId;
use alloc::sync::Arc;
use alloc::vec::Vec;
use node::Node;

mod node;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq)]
pub struct FlexTreeCore2<V> {
    upper_root: Arc<Node<V>>,
    lower_root: Arc<Node<V>>,
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
            upper_root: Node::empty(),
            lower_root: Node::empty(),
        }
    }

    /// [FlexId]と値を挿入する。
    /// 既に値がある場合には上書きされる。
    pub fn insert(&mut self, target: FlexId, value: V) {
        let (root, root_flexid) = self.root_for(&target);
        let written = Node::only_at(&root_flexid, &target, value);
        *root = Node::merge(&root_flexid, root, &written, &Node::overwrite_rule);
    }

    /// `target` の領域を空にする。
    pub fn remove(&mut self, target: FlexId) {
        let (root, root_flexid) = self.root_for(&target);
        let removed = Node::only_at(&root_flexid, &target, ());
        *root = Node::merge(&root_flexid, root, &removed, &Node::difference_rule);
    }

    /// 和集合。両方に値がある場所は `self` の値を使う。
    pub fn union(&self, other: &Self) -> Self {
        self.merge(other, &Node::union_rule)
    }

    /// 積集合。`other` にも値がある場所だけ、`self` の値を残す。
    pub fn intersection<W: Clone + PartialEq>(&self, other: &FlexTreeCore2<W>) -> Self {
        self.merge(other, &Node::intersection_rule)
    }

    /// 差集合。`other` に値がある場所を `self` から取り除く。
    pub fn difference<W: Clone + PartialEq>(&self, other: &FlexTreeCore2<W>) -> Self {
        self.merge(other, &Node::difference_rule)
    }

    /// 値を持つ全ての領域と値への参照を返す。
    pub fn iter(&self) -> impl Iterator<Item = (FlexId, &V)> {
        let mut out = Vec::new();
        for (root, id) in self.roots() {
            root.collect(&id, &mut out);
        }
        out.into_iter()
    }

    /// 上下のルートと領域 ID の組を返す。
    fn roots(&self) -> [(&Arc<Node<V>>, FlexId); 2] {
        [
            (&self.upper_root, FlexId::UPPER_MAX),
            (&self.lower_root, FlexId::LOWER_MAX),
        ]
    }

    /// 上下のルートどうしを `rule` で重ね合わせる。
    fn merge<W: Clone + PartialEq>(
        &self,
        other: &FlexTreeCore2<W>,
        rule: &impl Fn(&Arc<Node<V>>, &Arc<Node<W>>) -> Option<Arc<Node<V>>>,
    ) -> Self {
        FlexTreeCore2 {
            upper_root: Node::merge(
                &FlexId::UPPER_MAX,
                &self.upper_root,
                &other.upper_root,
                rule,
            ),
            lower_root: Node::merge(
                &FlexId::LOWER_MAX,
                &self.lower_root,
                &other.lower_root,
                rule,
            ),
        }
    }

    /// `target` が属する[Node]と、その[FlexId]を返す。
    /// 北半球と南半球の最初の分割用。
    fn root_for(&mut self, target: &FlexId) -> (&mut Arc<Node<V>>, FlexId) {
        if target.f_index().is_negative() {
            (&mut self.lower_root, FlexId::LOWER_MAX)
        } else {
            (&mut self.upper_root, FlexId::UPPER_MAX)
        }
    }
}
