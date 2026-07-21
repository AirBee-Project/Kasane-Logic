use super::FillEmpty;
use crate::{SpatialIdCollection, spatial_id::collection::query::execution::Query};

impl<S: SpatialIdCollection> Query<S>
where
    S::Value: 'static,
{
    /// コレクションを包む最小の[RangeId]内の空領域をデフォルト値で埋める単項演算子。
    pub fn fill_empty(self, default_value: S::Value) -> Self {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        let op = FillEmpty::new(default_value);
        self.wrap_unary(op)
    }
}
