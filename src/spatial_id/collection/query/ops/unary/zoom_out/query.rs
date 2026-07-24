use super::ZoomOut;
use crate::spatial_id::collection::query::merge_policy::MergePolicy;
use crate::spatial_id::collection::query::traits::WorkingTree;
use crate::spatial_id::{collection::query::execution::Query, zoom_level::ZoomLevel};

impl<W: WorkingTree + 'static> Query<W>
where
    W::Value: 'static,
{
    /// 指定されたズームレベルまで情報を落とし、複数の子ボクセルを`MergePolicy::resolve_many` で一括マージする単項演算子。
    pub fn zoom_out<P: MergePolicy<W::Value>, Z: Into<u8>>(self, target_z: Z, _policy: P) -> Self {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        match ZoomLevel::new(target_z.into()) {
            Ok(v) => {
                let op = ZoomOut::<W::Value, P>::new(v);
                self.wrap_unary(op)
            }
            Err(e) => Query::Error(e),
        }
    }
}
