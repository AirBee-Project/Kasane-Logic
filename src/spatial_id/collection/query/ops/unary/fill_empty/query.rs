use super::FillEmpty;
use crate::{SpatialIdCollection, spatial_id::collection::query::execution::Query};
use alloc::boxed::Box;

impl<S: SpatialIdCollection> Query<S>
where
    S::Value: 'static,
{
    /// テーブルを包む最小の RangeId（バウンディングボックス）内の空領域をデフォルト値で埋める。
    pub fn fill_empty(self, default_value: S::Value) -> Self {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        let op = FillEmpty::new(default_value);
        Query::Unary(Box::new(op), Box::new(self))
    }
}
