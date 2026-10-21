use super::{FilterValues, ValuePredicate};
use crate::query::SafeValue;
use crate::query::{Node, Query};

impl<V: SafeValue + Ord + 'static> Query<V> {
    /// 指定した値を持つ空間IDだけを残す。
    pub fn filter_eq(self, value: V) -> Self {
        self.filter_by(ValuePredicate::Equals(value))
    }

    /// 範囲に入る空間IDだけを残す。
    pub fn filter_in<R>(self, range: R) -> Self
    where
        R: core::ops::RangeBounds<V>,
        V: Clone,
    {
        self.filter_by(ValuePredicate::InRange(
            range.start_bound().cloned(),
            range.end_bound().cloned(),
        ))
    }

    /// 範囲に入る空間IDを取り除く（範囲外だけを残す）。
    pub fn filter_not_in<R>(self, range: R) -> Self
    where
        R: core::ops::RangeBounds<V>,
        V: Clone,
    {
        self.filter_by(ValuePredicate::NotInRange(
            range.start_bound().cloned(),
            range.end_bound().cloned(),
        ))
    }

    fn filter_by(self, predicate: ValuePredicate<V>) -> Self {
        if matches!(self.0, Node::Error(_)) {
            return self;
        }
        self.wrap_unary(FilterValues::new(predicate))
    }
}
