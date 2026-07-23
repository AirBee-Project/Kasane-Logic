use super::{FilterValues, ValuePredicate};
use crate::spatial_id::collection::query::{execution::Query, traits::WorkingTree};

impl<W: WorkingTree + 'static> Query<W>
where
    W::Value: Ord + 'static,
{
    /// 指定した値を持つセルだけを残す。
    pub fn filter_eq(self, value: W::Value) -> Self {
        self.filter_values(ValuePredicate::Equals(value))
    }

    /// 範囲に入るセルだけを残す。標準的な `RangeBounds` を受け取る（例: `1..=5`）。
    pub fn filter_in<R>(self, range: R) -> Self
    where
        R: core::ops::RangeBounds<W::Value>,
        W::Value: Clone,
    {
        self.filter_values(ValuePredicate::InRange(
            range.start_bound().cloned(),
            range.end_bound().cloned(),
        ))
    }

    /// 範囲に入るセルを取り除く（範囲外だけを残す）。標準的な `RangeBounds` を受け取る（例: `1..=5`）。
    pub fn filter_not_in<R>(self, range: R) -> Self
    where
        R: core::ops::RangeBounds<W::Value>,
        W::Value: Clone,
    {
        self.filter_values(ValuePredicate::NotInRange(
            range.start_bound().cloned(),
            range.end_bound().cloned(),
        ))
    }

    /// 値の条件でセルを絞り込む。
    pub fn filter_values(self, predicate: ValuePredicate<W::Value>) -> Self {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        self.wrap_unary(FilterValues::new(predicate))
    }
}
