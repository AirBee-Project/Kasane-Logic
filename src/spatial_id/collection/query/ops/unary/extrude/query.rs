use super::{extrude_f::ExtrudeF, extrude_x::ExtrudeX, extrude_y::ExtrudeY};
use crate::spatial_id::collection::query::traits::WorkingTree;
use crate::{
    ZoomLevel,
    spatial_id::collection::query::{execution::Query, merge_policy::MergePolicy},
};

impl<W: WorkingTree + 'static> Query<W>
where
    W::Value: 'static,
{
    /// X方向の Extrude (絶対座標による引き延ばし) 演算を適用する
    pub fn extrude_x<T: Into<u8>, P>(self, z: T, start_x: u32, end_x: u32, _policy: P) -> Self
    where
        P: MergePolicy<W::Value> + Send + Sync,
    {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        match ZoomLevel::new(z.into()) {
            Ok(zl) => self.wrap_unary(ExtrudeX::<P>::new(zl, start_x, end_x)),
            Err(e) => Query::Error(e),
        }
    }

    /// Y方向の Extrude (絶対座標による引き延ばし) 演算を適用する
    pub fn extrude_y<T: Into<u8>, P>(self, z: T, start_y: u32, end_y: u32, _policy: P) -> Self
    where
        P: MergePolicy<W::Value> + Send + Sync,
    {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        match ZoomLevel::new(z.into()) {
            Ok(zl) => self.wrap_unary(ExtrudeY::<P>::new(zl, start_y, end_y)),
            Err(e) => Query::Error(e),
        }
    }

    /// F方向の Extrude (絶対座標による引き延ばし) 演算を適用する
    pub fn extrude_f<T: Into<u8>, P>(self, z: T, start_f: i32, end_f: i32, _policy: P) -> Self
    where
        P: MergePolicy<W::Value> + Send + Sync,
    {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        match ZoomLevel::new(z.into()) {
            Ok(zl) => self.wrap_unary(ExtrudeF::<P>::new(zl, start_f, end_f)),
            Err(e) => Query::Error(e),
        }
    }
}
