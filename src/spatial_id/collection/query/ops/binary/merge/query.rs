use super::Merge;
use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::query::execution::Query;
use crate::spatial_id::collection::query::merge_policy::MergePolicy;
use alloc::boxed::Box;

impl<V: SafeValue + 'static> Query<V> {
    /// 2つのクエリ連鎖の結果を `MergePolicy` で重ね合わせる。
    /// 片側にしか値が無いSegmentは `default` を相手側の値とみなして解決する
    /// 両側とも値の無いSegmentはそのまま空となる。
    pub fn merge<P: MergePolicy<V>>(self, other: Self, default: V, _policy: P) -> Self {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        if matches!(other, Query::Error(_)) {
            return other;
        }
        let op = Merge::<V, P>::new(default);
        Query::Binary(Box::new(op), Box::new(self), Box::new(other))
    }
}
