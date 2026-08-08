use crate::spatial_id::collection::flex_tree::core::FlexTreeCore;
use alloc::vec::Vec;

use super::node::Node;
use super::node_ops::join_at;
use super::ptr::{SafeValue, SharedNode};
use super::split_child_id;
use crate::{FlexId, Side, SingleId};

/// 葉ノードを参照のまま辿るイテレータ。所有権を持つ [`FlexTreeCore::iter`] は
/// これを `map(clone)` して構築するため、走査ロジックはここ 1 か所に集約されている。
pub struct LeavesIterRef<'a, V>
where
    V: SafeValue,
{
    pub stack: Vec<(&'a Node<V>, FlexId)>,
}

/// 葉ノードの所有権を消費して辿るイテレータ。`Rc`/`Arc` が一意なら値をムーブで取り出す。
pub struct LeavesIntoIter<V>
where
    V: SafeValue,
{
    pub stack: Vec<(SharedNode<Node<V>>, FlexId)>,
}

/// Branch を降りるとき、下・上の子を ID 付きでスタックへ積む（葉走査の共通処理）。
#[inline]
fn push_children<T>(
    stack: &mut Vec<(T, FlexId)>,
    level: u8,
    current_id: &FlexId,
    lower: T,
    upper: T,
) {
    let axis = Node::<()>::axis(level);
    stack.push((upper, split_child_id(current_id, axis, Side::Upper)));
    stack.push((lower, split_child_id(current_id, axis, Side::Lower)));
}

impl<'a, V> Iterator for LeavesIterRef<'a, V>
where
    V: SafeValue,
{
    type Item = (FlexId, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((node, current_id)) = self.stack.pop() {
            match node {
                Node::Branch {
                    level,
                    lower_child,
                    upper_child,
                    ..
                } => push_children(
                    &mut self.stack,
                    *level,
                    &current_id,
                    lower_child.as_ref(),
                    upper_child.as_ref(),
                ),
                Node::Leaf { value: Some(value) } => return Some((current_id, value)),
                Node::Leaf { value: None } => {}
            }
        }
        None
    }
}

impl<V> Iterator for LeavesIntoIter<V>
where
    V: SafeValue,
{
    type Item = (FlexId, V);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((node_ptr, current_id)) = self.stack.pop() {
            let node = match super::ptr::try_unwrap(node_ptr) {
                Ok(node) => node,
                Err(shared) => (*shared).clone(),
            };

            match node {
                Node::Branch {
                    level,
                    lower_child,
                    upper_child,
                    ..
                } => push_children(
                    &mut self.stack,
                    level,
                    &current_id,
                    lower_child,
                    upper_child,
                ),
                Node::Leaf { value: Some(value) } => return Some((current_id, value)),
                Node::Leaf { value: None } => {}
            }
        }
        None
    }
}

impl<V> FlexTreeCore<V>
where
    V: SafeValue,
{
    pub fn single_ids(&self) -> impl Iterator<Item = SingleId> + '_ {
        self.iter()
            .flat_map(|(flex_id, _value)| flex_id.single_ids())
    }

    /// 木の**形はそのままに**、葉の値だけを別の型 `W` へ写した木を返す。
    ///
    /// `SpatialIdTable` の出入口（実体値 ⇄ ランク）のように、値を 1 対 1 で置き換える
    /// だけの変換のための経路。従来はいったん `Vec<(FlexId, W)>` へ平坦化してから木を
    /// 組み直していたが、形が変わらないのなら組み直す必要はない。ノード数ぶんの
    /// コピーで済み、葉ごとに根から降り直すコストが丸ごと消える。
    ///
    /// # `f` は単射でなければならない
    ///
    /// 正規形（FXY-正規形）は「値が等しい兄弟は畳む」で定まるので、`f` が異なる値を
    /// 同じ値へ潰すと、畳めるはずの枝が畳まれないまま残って正規形が壊れる。
    /// 単射なら値の等価・非等価がそのまま保たれるので、入力が正規形なら出力も正規形。
    pub(crate) fn map_values_injective<W, F>(&self, f: &F) -> FlexTreeCore<W>
    where
        W: SafeValue,
        F: Fn(&V) -> W + super::ptr::MaybeSync,
    {
        let empty_leaf = SharedNode::new(Node::Leaf { value: None });
        FlexTreeCore {
            lower_root: map_node(&self.lower_root, f, &empty_leaf),
            upper_root: map_node(&self.upper_root, f, &empty_leaf),
            empty_leaf,
            shard: self.shard,
            shard_path: self.shard_path.clone(),
        }
    }
}

/// [`map_values_injective`](FlexTreeCore::map_values_injective) の再帰本体。
fn map_node<V, W, F>(
    node: &SharedNode<Node<V>>,
    f: &F,
    empty_leaf: &SharedNode<Node<W>>,
) -> SharedNode<Node<W>>
where
    V: SafeValue,
    W: SafeValue,
    F: Fn(&V) -> W + super::ptr::MaybeSync,
{
    match &**node {
        Node::Leaf { value: None } => empty_leaf.clone(),
        Node::Leaf { value: Some(v) } => SharedNode::new(Node::Leaf { value: Some(f(v)) }),
        Node::Branch {
            level,
            leaf_count,
            max_zoom,
            split_mask,
            lower_child,
            upper_child,
        } => {
            // 形を写すだけなので、畳み込み判定もキャッシュの再計算も要らない
            // （`f` が単射なので畳める枝は増えも減りもしない）。
            let (lower, upper) = join_at!(
                super::node_ops::PARALLEL_LEAF_CUTOFF,
                *leaf_count as usize,
                || map_node(lower_child, f, empty_leaf),
                || map_node(upper_child, f, empty_leaf)
            );
            SharedNode::new(Node::Branch {
                level: *level,
                leaf_count: *leaf_count,
                max_zoom: *max_zoom,
                split_mask: *split_mask,
                lower_child: lower,
                upper_child: upper,
            })
        }
    }
}
