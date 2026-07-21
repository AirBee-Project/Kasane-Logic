use super::ZoomOut;
use crate::spatial_id::collection::query::merge_policy::MergePolicy;
use crate::{
    SpatialIdCollection,
    spatial_id::{collection::query::execution::Query, zoom_level::ZoomLevel},
};

impl<S: SpatialIdCollection> Query<S>
where
    S::Value: 'static,
{
    /// 指定されたズームレベルまで情報を落とし、複数の子ボクセルを`MergePolicy::resolve_many` で一括マージする単項演算子。
    pub fn zoom_out<P: MergePolicy<S::Value>, Z: Into<u8>>(self, target_z: Z, _policy: P) -> Self {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        match ZoomLevel::new(target_z.into()) {
            Ok(v) => {
                let op = ZoomOut::<S::Value, P>::new(v);
                self.wrap_unary(op)
            }
            Err(e) => Query::Error(e),
        }
    }
}
