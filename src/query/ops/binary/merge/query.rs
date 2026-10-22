use super::Merge;
use crate::query::SafeValue;
use crate::query::merge_policy::MergePolicy;
use crate::query::{Node, Query};
use alloc::boxed::Box;

impl<V: SafeValue + 'static> Query<V> {
    pub fn merge<P: MergePolicy<V>>(self, other: impl Into<Self>, _policy: P) -> Self {
        if matches!(self.0, Node::Error(_)) {
            return self;
        }
        let other = other.into();
        if matches!(other.0, Node::Error(_)) {
            return other;
        }
        let op = Merge::<V, P>::new();
        Query(Node::Binary(Box::new(op), Box::new(self), Box::new(other)))
    }
}
