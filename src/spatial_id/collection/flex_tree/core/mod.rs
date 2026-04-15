use crate::{
    Dimension, FlexId, IntoFlexIds, IterFlexIds,
    spatial_id::collection::flex_tree::core::convert::LeavesIter,
};
use node::Node;
mod convert;
mod node;
mod overlap;

/// 拡張空間IDとそれに紐づいたValueを保存するための型
#[derive(Clone, Default)]
pub struct FlexTreeCore<V>
where
    V: PartialEq + Clone,
{
    lower_root: Option<Box<Node<V>>>,
    upper_root: Option<Box<Node<V>>>,
}

impl<V> FlexTreeCore<V>
where
    V: PartialEq + Clone,
{
    /// 新しい空の[FlexTreeCore]を作成する
    pub fn new() -> Self {
        Self {
            lower_root: None,
            upper_root: None,
        }
    }

    ///クリアする
    pub fn clear(&mut self) {
        self.lower_root = None;
        self.upper_root = None;
    }

    pub fn is_empty(&self) -> bool {
        self.lower_root.is_none() && self.upper_root.is_none()
    }

    pub fn count(&self) -> usize {
        //Todo:型の内部に個数をキャッシュしてO(1)で求められるようにする
        self.iter().count()
    }

    /// [FlexTreeCore]に空間IDを挿入する
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

    /// [FlexTreeCore]からtargetと重なりがある[FlexId]とそのValueを全て取り出す
    pub fn get<'a, S>(&'a self, target: &'a S) -> impl Iterator<Item = (FlexId, V)> + 'a
    where
        S: IterFlexIds,
    {
        target
            .iter_flex_ids()
            .flat_map(move |item| self.overlap(item))
    }

    /// [FlexTreeCore]からTargetが示す領域を切り取って返す
    pub fn remove<S>(&mut self, target: &S) -> impl Iterator<Item = (FlexId, V)>
    where
        S: IterFlexIds,
    {
        let mut actual_removed = Vec::new();

        for t_id in target.iter_flex_ids() {
            let affected_leaves: Vec<(FlexId, V)> = self.overlap_remove(&t_id).collect();

            for (leaf_id, value) in affected_leaves {
                for remnant_id in leaf_id.difference(&t_id) {
                    self.insert(remnant_id, value.clone());
                }
                if let Some(intersect_id) = leaf_id.intersection(&t_id) {
                    actual_removed.push((intersect_id, value));
                }
            }
        }

        actual_removed.into_iter()
    }

    /// [FlexTreeCore]から全ての[FlexId]とValueを取り出す
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
