use crate::{
    Dimension, FlexId, FlexTreeCore, Side,
    spatial_id::collection::flex_tree::core::{node::Node, split_child_id},
};

/// 重なり合う領域のみを遅延評価で探索するイテレータ
pub struct OverlapIter<'a, V>
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
    /// 現在ノードと軸情報から次に探索する子ノードをスタックへ積む。
    fn push_branch_children(
        &mut self,
        axis: Dimension,
        lower_child: &'a Option<Box<Node<V>>>,
        upper_child: &'a Option<Box<Node<V>>>,
        current_id: &FlexId,
    ) {
        Self::push_child(
            &mut self.stack,
            axis,
            Side::Upper,
            upper_child.as_deref(),
            current_id,
        );
        Self::push_child(
            &mut self.stack,
            axis,
            Side::Lower,
            lower_child.as_deref(),
            current_id,
        );
    }

    /// 子ノードが存在する場合に、対応する分割 ID と一緒にスタックへ積む。
    fn push_child(
        stack: &mut Vec<(&'a Node<V>, FlexId)>,
        axis: Dimension,
        side: Side,
        child: Option<&'a Node<V>>,
        current_id: &FlexId,
    ) {
        if let Some(child) = child {
            stack.push((child, split_child_id(current_id, axis, side)));
        }
    }
}

impl<'a, V> Iterator for OverlapIter<'a, V>
where
    V: PartialEq + Clone,
{
    type Item = (FlexId, V);

    /// スタックを深さ優先でたどり、target と重なる葉ノードだけを順次返す。
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
                    self.push_branch_children(*axis, lower_child, upper_child, &current_id);
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
        OverlapIter {
            target,
            stack: self.root_node_stack(),
        }
    }

    ///Targetと重なっている[FlexId]とValueを全て削除し、取得する関数
    pub fn overlap_remove(&mut self, target: &FlexId) -> impl Iterator<Item = (FlexId, V)> {
        // 削除された要素を一時的に集めるためのベクタ
        let mut removed_items = Vec::new();

        Self::prune_root_nodes(self, target, &mut removed_items);
        removed_items.into_iter()
    }

    /// 上下ルートをまとめて走査し、対象と重なる葉ノードを削除して収集する。
    fn prune_root_nodes(this: &mut Self, target: &FlexId, removed: &mut Vec<(FlexId, V)>) {
        Self::prune_node(&mut this.lower_root, target, FlexId::LOWER_MAX, removed);
        Self::prune_node(&mut this.upper_root, target, FlexId::UPPER_MAX, removed);
    }

    /// 対象領域と重なる部分木のみを再帰的に剪定して削除要素を収集する。
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
        let should_keep = match node.as_mut() {
            Node::Branch {
                axis,
                lower_child,
                upper_child,
            } => Self::prune_branch_children(
                *axis,
                lower_child,
                upper_child,
                target,
                &current_id,
                removed,
            ),
            Node::Leaf { .. } => false,
        };

        Self::restore_or_collect(node_opt, node, should_keep, current_id, removed);
    }

    /// 分岐ノードの子を再帰的に剪定し、子が残っているかどうかを返す。
    fn prune_branch_children(
        axis: Dimension,
        lower_child: &mut Option<Box<Node<V>>>,
        upper_child: &mut Option<Box<Node<V>>>,
        target: &FlexId,
        current_id: &FlexId,
        removed: &mut Vec<(FlexId, V)>,
    ) -> bool {
        Self::prune_child(lower_child, axis, Side::Lower, target, current_id, removed);
        Self::prune_child(upper_child, axis, Side::Upper, target, current_id, removed);

        lower_child.is_some() || upper_child.is_some()
    }

    /// 指定した side の子ノードを一段進めた ID で再帰的に剪定する。
    fn prune_child(
        child: &mut Option<Box<Node<V>>>,
        axis: Dimension,
        side: Side,
        target: &FlexId,
        current_id: &FlexId,
        removed: &mut Vec<(FlexId, V)>,
    ) {
        if child.is_some() {
            let next_id = split_child_id(current_id, axis, side);
            Self::prune_node(child, target, next_id, removed);
        }
    }

    /// 剪定後にノードを戻すか、葉として削除結果へ収集するかを決定する。
    fn restore_or_collect(
        node_opt: &mut Option<Box<Node<V>>>,
        node: Box<Node<V>>,
        should_keep: bool,
        current_id: FlexId,
        removed: &mut Vec<(FlexId, V)>,
    ) {
        if should_keep {
            *node_opt = Some(node);
            return;
        }

        if let Node::Leaf { value } = *node {
            removed.push((current_id, value));
        }
    }
}
