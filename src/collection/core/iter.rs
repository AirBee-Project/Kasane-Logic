use alloc::vec::Vec;
use core::iter::FusedIterator;
use triomphe::Arc;

use super::FlexTreeCore;
use super::node::Node;
use crate::collection::summary::Summary;
use crate::{FlexId, Side, spatial_id::dimension::Dimension};

/// [FlexTreeCore]の値を持つ領域と値への参照を、Node を辿りながら1つずつ返すイテレーター。
#[derive(Debug)]
pub(crate) struct Iter<'a, V, S> {
    stack: Vec<(&'a Node<V, S>, FlexId)>,
    /// まだ返していない要素の数。
    remaining: usize,
}

impl<'a, V, S> Iter<'a, V, S> {
    pub(super) fn new(tree: &'a FlexTreeCore<V, S>) -> Self {
        Iter {
            stack: tree.new_stack(),
            remaining: tree.count(),
        }
    }
}

impl<V, S> Clone for Iter<'_, V, S> {
    fn clone(&self) -> Self {
        Iter {
            stack: self.stack.clone(),
            remaining: self.remaining,
        }
    }
}

impl<'a, V, S> Iterator for Iter<'a, V, S> {
    type Item = (FlexId, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((node, this)) = self.stack.pop() {
            if let Node::Leaf(value) = node {
                self.remaining -= 1;
                return Some((this, value));
            }
            self.stack.extend(node.children(this).rev());
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<V, S> ExactSizeIterator for Iter<'_, V, S> {}

impl<V, S> FusedIterator for Iter<'_, V, S> {}

/// 値を持つ領域と値の組 `(FlexId, V)` を、所有権ごと1つずつ返すイテレーター。
///
/// [`SpatialIdMap::remove`](crate::SpatialIdMap::remove) や、[`SpatialIdMap`](crate::SpatialIdMap) の `into_iter` が返す。
#[derive(Debug)]
pub struct IntoIter<V, S> {
    stack: Vec<(Node<V, S>, FlexId)>,
    /// まだ返していない要素の数。
    remaining: usize,
}

impl<V: Clone, S> Iterator for IntoIter<V, S> {
    type Item = (FlexId, V);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((node, this)) = self.stack.pop() {
            match node {
                Node::Empty => {}
                Node::Leaf(value) => {
                    self.remaining -= 1;
                    return Some((this, value));
                }
                Node::Branch(branch) => match Arc::try_unwrap(branch) {
                    Ok(branch) => {
                        self.push_branch(this, branch.dimension, branch.lower, branch.upper)
                    }
                    Err(branch) => self.push_branch(
                        this,
                        branch.dimension,
                        branch.lower.clone(),
                        branch.upper.clone(),
                    ),
                },
                Node::Skip(skip) => {
                    let region = skip.path.to_absolute(&this).unwrap();
                    let child = Arc::try_unwrap(skip)
                        .map_or_else(|skip| skip.child.clone(), |skip| skip.child);
                    self.stack.push((child, region));
                }
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<V, S> IntoIter<V, S> {
    /// 領域 `this` を `dimension` で割った子を、[Iter]と同じく下側から取り出されるように積む。
    fn push_branch(
        &mut self,
        this: FlexId,
        dimension: Dimension,
        lower: Node<V, S>,
        upper: Node<V, S>,
    ) {
        self.stack
            .push((upper, this.split_on(dimension, Side::Upper).unwrap()));
        self.stack
            .push((lower, this.split_on(dimension, Side::Lower).unwrap()));
    }
}

impl<V: Clone, S> ExactSizeIterator for IntoIter<V, S> {}

impl<V: Clone, S> FusedIterator for IntoIter<V, S> {}

impl<'a, V, S> IntoIterator for &'a FlexTreeCore<V, S> {
    type Item = (FlexId, &'a V);
    type IntoIter = Iter<'a, V, S>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<V: Clone, S> IntoIterator for FlexTreeCore<V, S> {
    type Item = (FlexId, V);
    type IntoIter = IntoIter<V, S>;

    fn into_iter(self) -> Self::IntoIter {
        let remaining = self.count();
        let height = self.upper_root.height().max(self.lower_root.height());
        let mut stack = Vec::with_capacity(usize::from(height) + 2);
        // 後に積んだものから取り出すので、上側のルートを後に積む
        stack.push((self.lower_root, FlexId::LOWER_MAX));
        stack.push((self.upper_root, FlexId::UPPER_MAX));
        IntoIter { stack, remaining }
    }
}

impl<V: PartialEq + Clone, S: Summary<V>> Extend<(FlexId, V)> for FlexTreeCore<V, S> {
    /// 順に [`insert`](FlexTreeCore::insert) する。重なる場所は後の値で上書きされる。
    fn extend<I: IntoIterator<Item = (FlexId, V)>>(&mut self, iter: I) {
        for (id, value) in iter {
            self.insert(id, value);
        }
    }
}

/// FlexId の Z-order 列（`prev < next`）から、1つのルート配下の Node を下から O(n) で組む。
struct OrderedRootBuilder<V, S> {
    root_id: FlexId,
    prev_id: FlexId,
    cur_region: FlexId,
    cur_node: Node<V, S>,
    stack: Vec<(FlexId, Dimension, u8, Node<V, S>)>,
}

impl<V: PartialEq + Clone, S: Summary<V>> OrderedRootBuilder<V, S> {
    fn new(root_id: FlexId, id: FlexId, value: V) -> Self {
        Self {
            root_id,
            prev_id: id,
            cur_region: id,
            cur_node: Node::Leaf(value),
            stack: Vec::new(),
        }
    }

    fn try_push(&mut self, id: FlexId, value: V) -> Result<(), (FlexId, V)> {
        let Some((fork, split_dim, agree)) = self.prev_id.zorder_fork(&id) else {
            return Err((id, value));
        };
        while let Some(&(top_fork, top_dim, top_agree, _)) = self.stack.last() {
            if (top_agree, top_dim) <= (agree, split_dim) {
                break;
            }
            let (_, _, _, lower) = self.stack.pop().unwrap();
            let upper_id = top_fork.split_on(top_dim, Side::Upper).unwrap();
            let upper = Node::rebase(
                core::mem::take(&mut self.cur_node),
                &self.cur_region,
                &upper_id,
            );
            self.cur_node = Node::join(&top_fork, top_dim, lower, upper);
            self.cur_region = top_fork;
        }
        let lower_id = fork.split_on(split_dim, Side::Lower).unwrap();
        let lower = Node::rebase(
            core::mem::take(&mut self.cur_node),
            &self.cur_region,
            &lower_id,
        );
        self.stack.push((fork, split_dim, agree, lower));
        self.prev_id = id;
        self.cur_region = id;
        self.cur_node = Node::Leaf(value);
        Ok(())
    }

    fn finish(mut self) -> Node<V, S> {
        while let Some((top_fork, top_dim, _, lower)) = self.stack.pop() {
            let upper_id = top_fork.split_on(top_dim, Side::Upper).unwrap();
            let upper = Node::rebase(self.cur_node, &self.cur_region, &upper_id);
            self.cur_node = Node::join(&top_fork, top_dim, lower, upper);
            self.cur_region = top_fork;
        }
        Node::rebase(self.cur_node, &self.cur_region, &self.root_id)
    }
}

fn finish_builders<V: PartialEq + Clone, S: Summary<V>>(
    upper: Option<OrderedRootBuilder<V, S>>,
    lower: Option<OrderedRootBuilder<V, S>>,
) -> FlexTreeCore<V, S> {
    FlexTreeCore {
        upper_root: upper.map_or(Node::Empty, OrderedRootBuilder::finish),
        lower_root: lower.map_or(Node::Empty, OrderedRootBuilder::finish),
    }
}

impl<V: PartialEq + Clone, S: Summary<V>> FromIterator<(FlexId, V)> for FlexTreeCore<V, S> {
    /// FlexId の Z-order 列なら下から O(n) で組み立て、途中で順序が崩れていれば [`insert`](FlexTreeCore::insert) に切り替える。
    fn from_iter<I: IntoIterator<Item = (FlexId, V)>>(iter: I) -> Self {
        let mut iter = iter.into_iter();
        let mut upper: Option<OrderedRootBuilder<V, S>> = None;
        let mut lower: Option<OrderedRootBuilder<V, S>> = None;

        for (id, value) in iter.by_ref() {
            let root_idx = id.root_index();
            let slot = if root_idx == 0 {
                if lower.is_some() {
                    let mut tree = finish_builders(upper, lower);
                    tree.insert(id, value);
                    tree.extend(iter);
                    return tree;
                }
                &mut upper
            } else {
                &mut lower
            };

            match slot {
                None => *slot = Some(OrderedRootBuilder::new(FlexId::ROOTS[root_idx], id, value)),
                Some(builder) => {
                    if let Err((id, value)) = builder.try_push(id, value) {
                        let mut tree = finish_builders(upper, lower);
                        tree.insert(id, value);
                        tree.extend(iter);
                        return tree;
                    }
                }
            }
        }

        finish_builders(upper, lower)
    }
}

/// スレッドごとに部分木を組み、和集合で畳む（`feature = "rayon"`）。
///
/// 同じ場所へ異なる値が重なった場合にどちらが残るかは、チャンクの分かれ方で決まり、
/// 逐次の [`FromIterator`] の後勝ちとは一致しない。
#[cfg(feature = "rayon")]
impl<V, S> rayon::iter::FromParallelIterator<(FlexId, V)> for FlexTreeCore<V, S>
where
    V: PartialEq + Clone + Send + Sync,
    S: Summary<V> + Send + Sync,
{
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: rayon::iter::IntoParallelIterator<Item = (FlexId, V)>,
    {
        use rayon::prelude::*;
        par_iter
            .into_par_iter()
            .fold(FlexTreeCore::default, |mut tree, (id, value)| {
                tree.insert(id, value);
                tree
            })
            .reduce(FlexTreeCore::default, |a, b| a.union(&b))
    }
}

/// 並列に組んだ木を和集合で重ねる（`feature = "rayon"`）。重なる場所は追加する側の値になる。
#[cfg(feature = "rayon")]
impl<V, S> rayon::iter::ParallelExtend<(FlexId, V)> for FlexTreeCore<V, S>
where
    V: PartialEq + Clone + Send + Sync,
    S: Summary<V> + Send + Sync,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: rayon::iter::IntoParallelIterator<Item = (FlexId, V)>,
    {
        use rayon::iter::FromParallelIterator;
        *self = Self::from_par_iter(par_iter).union(self);
    }
}
