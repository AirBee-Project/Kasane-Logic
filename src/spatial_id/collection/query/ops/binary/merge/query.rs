use super::Merge;
use crate::spatial_id::collection::query::merge_policy::MergePolicy;
use crate::{SpatialIdCollection, spatial_id::collection::query::execution::Query};
use alloc::boxed::Box;

impl<S: SpatialIdCollection> Query<S>
where
    S::Value: 'static,
{
    /// 2つのクエリ連鎖の結果を `MergePolicy` で重ね合わせる。
    /// 片側にしか値が無いセルは `default` を相手側の値とみなして解決する
    /// （両側とも値の無いセルはそのまま空）。
    pub fn merge<P: MergePolicy<S::Value>>(
        self,
        other: Self,
        default: S::Value,
        _policy: P,
    ) -> Self {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        if matches!(other, Query::Error(_)) {
            return other;
        }
        let op = Merge::<S::Value, P>::new(default);
        Query::Binary(Box::new(op), Box::new(self), Box::new(other))
    }
}
