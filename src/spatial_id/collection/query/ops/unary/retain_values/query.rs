use super::{RetainValues, ValuePredicate};
use crate::spatial_id::collection::query::{execution::Query, traits::WorkingTree};

impl<W: WorkingTree + 'static> Query<W>
where
    W::Value: Ord + 'static,
{
    /// 指定した値を持つセルだけを残す。
    pub fn retain_value_eq(self, value: W::Value) -> Self {
        self.retain_values(ValuePredicate::Equals(value))
    }

    /// 値が `min..=max`（閉区間）に入るセルだけを残す。境界の `None` はその側を無制限にする。
    pub fn retain_value_in_range(self, min: Option<W::Value>, max: Option<W::Value>) -> Self {
        self.retain_values(ValuePredicate::InRange { min, max })
    }

    /// 値が `min..=max`（閉区間）に入るセルを取り除く（範囲外だけを残す）。
    pub fn retain_value_not_in_range(self, min: Option<W::Value>, max: Option<W::Value>) -> Self {
        self.retain_values(ValuePredicate::NotInRange { min, max })
    }

    /// 値の条件でセルを絞り込む。
    pub fn retain_values(self, predicate: ValuePredicate<W::Value>) -> Self {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        self.wrap_unary(RetainValues::new(predicate))
    }
}
