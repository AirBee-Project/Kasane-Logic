use super::Merge;
use crate::spatial_id::collection::query::merge_policy::MergePolicy;
use crate::spatial_id::collection::query::{execution::Query, traits::WorkingTree};
use alloc::boxed::Box;

impl<W: WorkingTree + 'static> Query<W>
where
    W::Value: 'static,
{
    /// 2つのクエリ連鎖の結果を `MergePolicy` で重ね合わせる。
    /// 片側にしか値が無いセルは `default` を相手側の値とみなして解決する
    /// 両側とも値の無いセルはそのまま空となる。
    pub fn merge<Q, P>(self, other: Q, default: W::Value, _policy: P) -> Self
    where
        Q: Into<Query<W>>,
        P: MergePolicy<W::Value>,
    {
        let other = other.into();
        if matches!(self, Query::Error(_)) {
            return self;
        }
        if matches!(other, Query::Error(_)) {
            return other;
        }
        let op = Merge::<W::Value, P>::new(default);
        Query::Binary(Box::new(op), Box::new(self), Box::new(other))
    }
}
