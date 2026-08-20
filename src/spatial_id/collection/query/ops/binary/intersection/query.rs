use super::Intersection;
use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::query::execution::Query;
use alloc::boxed::Box;

impl<V: SafeValue + 'static> Query<V> {
    pub fn intersection(self, other: Self) -> Self {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        if matches!(other, Query::Error(_)) {
            return other;
        }
        let op = Intersection::<V>::new();
        Query::Binary(Box::new(op), Box::new(self), Box::new(other))
    }
}
