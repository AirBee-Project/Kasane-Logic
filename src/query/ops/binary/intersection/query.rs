use super::Intersection;
use crate::query::SafeValue;
use crate::query::{Node, Query};
use alloc::boxed::Box;

impl<V: SafeValue + 'static> Query<V> {
    pub fn intersection(self, other: impl Into<Self>) -> Self {
        if matches!(self.0, Node::Error(_)) {
            return self;
        }
        let other = other.into();
        if matches!(other.0, Node::Error(_)) {
            return other;
        }
        let op = Intersection::<V>::new();
        Query(Node::Binary(Box::new(op), Box::new(self), Box::new(other)))
    }
}
