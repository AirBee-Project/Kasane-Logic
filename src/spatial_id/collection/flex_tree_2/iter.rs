use alloc::sync::Arc;
use alloc::vec::Vec;
use core::iter::FusedIterator;

use super::FlexTreeCore2;
use super::node::Node;
use crate::{FlexId, Side, spatial_id::dimension::Dimension};

/// [FlexTreeCore2]の値を持つ領域と値への参照を、木を辿りながら1つずつ返すイテレーター。
#[derive(Debug, Clone)]
pub struct Iter<'a, V> {
    stack: Vec<(&'a Node<V>, FlexId)>,
}

impl<'a, V> Iter<'a, V> {
    pub(super) fn new(tree: &'a FlexTreeCore2<V>) -> Self {
        // 後に積んだものから取り出すので、上側のルートを後に積む
        Iter {
            stack: Vec::from([
                (&*tree.lower_root, FlexId::LOWER_MAX),
                (&*tree.upper_root, FlexId::UPPER_MAX),
            ]),
        }
    }
}

impl<'a, V> Iterator for Iter<'a, V> {
    type Item = (FlexId, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((node, this)) = self.stack.pop() {
            match node {
                Node::Leaf(None) => {}
                Node::Leaf(Some(value)) => return Some((this, value)),
                Node::Branch {
                    dimension,
                    lower,
                    upper,
                    ..
                } => {
                    // 下側を先に返すため、上側を先に積む
                    self.stack
                        .push((upper, this.split_on(*dimension, Side::Upper).unwrap()));
                    self.stack
                        .push((lower, this.split_on(*dimension, Side::Lower).unwrap()));
                }
                Node::Skip { path, child, .. } => {
                    self.stack.push((child, path.to_absolute(&this).unwrap()));
                }
            }
        }
        None
    }
}

impl<V> FusedIterator for Iter<'_, V> {}

/// [FlexTreeCore2]を消費して、値を持つ領域と値を1つずつ返すイテレーター。
/// ノードを他の木と共有していなければ値をムーブし、共有していればクローンする。
#[derive(Debug)]
pub struct IntoIter<V> {
    stack: Vec<(Arc<Node<V>>, FlexId)>,
}

impl<V: Clone> Iterator for IntoIter<V> {
    type Item = (FlexId, V);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((node, this)) = self.stack.pop() {
            match Arc::try_unwrap(node) {
                Ok(Node::Leaf(Some(value))) => return Some((this, value)),
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
                    Node::Leaf(Some(value)) => return Some((this, value.clone())),
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
}

impl<V> IntoIter<V> {
    /// 領域 `this` を `dimension` で割った子を、[Iter]と同じく下側から取り出されるように積む。
    fn push_branch(
        &mut self,
        this: FlexId,
        dimension: Dimension,
        lower: Arc<Node<V>>,
        upper: Arc<Node<V>>,
    ) {
        self.stack
            .push((upper, this.split_on(dimension, Side::Upper).unwrap()));
        self.stack
            .push((lower, this.split_on(dimension, Side::Lower).unwrap()));
    }
}

impl<V: Clone> FusedIterator for IntoIter<V> {}

impl<'a, V: Clone + Ord> IntoIterator for &'a FlexTreeCore2<V> {
    type Item = (FlexId, &'a V);
    type IntoIter = Iter<'a, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<V: Clone + Ord> IntoIterator for FlexTreeCore2<V> {
    type Item = (FlexId, V);
    type IntoIter = IntoIter<V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            stack: Vec::from([
                (self.lower_root, FlexId::LOWER_MAX),
                (self.upper_root, FlexId::UPPER_MAX),
            ]),
        }
    }
}

impl<V: Clone + Ord> Extend<(FlexId, V)> for FlexTreeCore2<V> {
    /// 順に [`insert`](FlexTreeCore2::insert) する。重なる場所は後の値で上書きされる。
    fn extend<I: IntoIterator<Item = (FlexId, V)>>(&mut self, iter: I) {
        for (id, value) in iter {
            self.insert(id, value);
        }
    }
}

impl<V: Clone + Ord> FromIterator<(FlexId, V)> for FlexTreeCore2<V> {
    /// 順に [`insert`](FlexTreeCore2::insert) して組み立てる。重なる場所は後の値で上書きされる。
    fn from_iter<I: IntoIterator<Item = (FlexId, V)>>(iter: I) -> Self {
        let mut tree = FlexTreeCore2::new();
        tree.extend(iter);
        tree
    }
}
