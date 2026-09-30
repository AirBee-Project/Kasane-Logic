use alloc::sync::Arc;
use alloc::vec::Vec;
use core::iter::FusedIterator;

use super::FlexTreeCore2;
use super::node::Node;
use super::summary::Summary;
use crate::{FlexId, Side, spatial_id::dimension::Dimension};

/// [FlexTreeCore2]の値を持つ領域と値への参照を、Node を辿りながら1つずつ返すイテレーター。
#[derive(Debug)]
pub struct Iter<'a, V, S> {
    stack: Vec<(&'a Node<V, S>, FlexId)>,
    /// まだ返していない要素の数。
    remaining: usize,
}

impl<'a, V, S> Iter<'a, V, S> {
    pub(super) fn new(tree: &'a FlexTreeCore2<V, S>) -> Self {
        Iter {
            stack: tree.new_stack(),
            remaining: tree.len(),
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
            if let Node::Leaf(Some(value)) = node {
                self.remaining -= 1;
                return Some((this, value));
            }
            node.push_children(this, &mut self.stack);
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<V, S> ExactSizeIterator for Iter<'_, V, S> {}

impl<V, S> FusedIterator for Iter<'_, V, S> {}

/// [FlexTreeCore2]を消費して、値を持つ領域と値を1つずつ返すイテレーター。
#[derive(Debug)]
pub struct IntoIter<V, S> {
    stack: Vec<(Arc<Node<V, S>>, FlexId)>,
    /// まだ返していない要素の数。
    remaining: usize,
}

impl<V: Clone, S> Iterator for IntoIter<V, S> {
    type Item = (FlexId, V);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((node, this)) = self.stack.pop() {
            match Arc::try_unwrap(node) {
                Ok(Node::Leaf(Some(value))) => {
                    self.remaining -= 1;
                    return Some((this, value));
                }
                Ok(Node::Leaf(None)) => {}
                Ok(Node::Branch {
                    dimension,
                    lower,
                    upper,
                    ..
                }) => self.push_branch(this, dimension, lower, upper),
                Ok(Node::Skip { path, child, .. }) => {
                    self.stack.push((child, path.to_absolute(&this).unwrap()));
                }
                Err(shared) => match &*shared {
                    Node::Leaf(Some(value)) => {
                        self.remaining -= 1;
                        return Some((this, value.clone()));
                    }
                    Node::Leaf(None) => {}
                    Node::Branch {
                        dimension,
                        lower,
                        upper,
                        ..
                    } => self.push_branch(this, *dimension, lower.clone(), upper.clone()),
                    Node::Skip { path, child, .. } => {
                        self.stack
                            .push((child.clone(), path.to_absolute(&this).unwrap()));
                    }
                },
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
        lower: Arc<Node<V, S>>,
        upper: Arc<Node<V, S>>,
    ) {
        self.stack
            .push((upper, this.split_on(dimension, Side::Upper).unwrap()));
        self.stack
            .push((lower, this.split_on(dimension, Side::Lower).unwrap()));
    }
}

impl<V: Clone, S> ExactSizeIterator for IntoIter<V, S> {}

impl<V: Clone, S> FusedIterator for IntoIter<V, S> {}

impl<'a, V, S> IntoIterator for &'a FlexTreeCore2<V, S> {
    type Item = (FlexId, &'a V);
    type IntoIter = Iter<'a, V, S>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<V: Clone, S> IntoIterator for FlexTreeCore2<V, S> {
    type Item = (FlexId, V);
    type IntoIter = IntoIter<V, S>;

    fn into_iter(self) -> Self::IntoIter {
        let remaining = self.len();
        let height = self.upper_root.height().max(self.lower_root.height());
        let mut stack = Vec::with_capacity(usize::from(height) + 2);
        // 後に積んだものから取り出すので、上側のルートを後に積む
        stack.push((self.lower_root, FlexId::LOWER_MAX));
        stack.push((self.upper_root, FlexId::UPPER_MAX));
        IntoIter { stack, remaining }
    }
}

impl<V: PartialEq, S: Summary<V>> Extend<(FlexId, V)> for FlexTreeCore2<V, S> {
    /// 順に [`insert`](FlexTreeCore2::insert) する。重なる場所は後の値で上書きされる。
    fn extend<I: IntoIterator<Item = (FlexId, V)>>(&mut self, iter: I) {
        for (id, value) in iter {
            self.insert(id, value);
        }
    }
}

impl<V: PartialEq, S: Summary<V>> FromIterator<(FlexId, V)> for FlexTreeCore2<V, S> {
    /// 順に [`insert`](FlexTreeCore2::insert) して組み立てる。重なる場所は後の値で上書きされる。
    fn from_iter<I: IntoIterator<Item = (FlexId, V)>>(iter: I) -> Self {
        let mut tree = FlexTreeCore2::default();
        tree.extend(iter);
        tree
    }
}
