use super::{FilterValues, ValuePredicate};
use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::query::execution::Query;

impl<V: SafeValue + Ord + 'static> Query<V> {
    /// 指定した値を持つ空間IDだけを残す。
    pub fn filter_eq(self, value: V) -> Self {
        self.filter_values(ValuePredicate::Equals(value))
    }

    /// 範囲に入る空間IDだけを残す。
    pub fn filter_in<R>(self, range: R) -> Self
    where
        R: core::ops::RangeBounds<V>,
        V: Clone,
    {
        self.filter_values(ValuePredicate::InRange(
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
        self.filter_values(ValuePredicate::NotInRange(
            range.start_bound().cloned(),
            range.end_bound().cloned(),
        ))
    }

    /// 値の条件で空間IDを絞り込む。
    pub fn filter_values(self, predicate: ValuePredicate<V>) -> Self {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        self.wrap_unary(FilterValues::new(predicate))
    }
}
