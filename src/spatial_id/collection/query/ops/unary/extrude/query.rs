use super::{
    extrude_f::ExtrudeF, extrude_fxy::ExtrudeFXY, extrude_x::ExtrudeX, extrude_y::ExtrudeY,
};
use crate::{
    SpatialIdCollection, ZoomLevel,
    spatial_id::collection::query::{execution::Query, merge_policy::MergePolicy},
};
use alloc::boxed::Box;

impl<S: SpatialIdCollection> Query<S>
where
    S::Value: 'static,
{
    /// X方向の Extrude (絶対座標による引き延ばし) 演算を適用する
    pub fn extrude_x<T: Into<u8>, P>(self, z: T, start_x: u32, end_x: u32, _policy: P) -> Self
    where
        P: MergePolicy<S::Value> + Send + Sync,
    {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        match ZoomLevel::new(z.into()) {
            Ok(zl) => Query::Unary(
                Box::new(ExtrudeX::<P>::new(zl, start_x, end_x)),
                Box::new(self),
            ),
            Err(e) => Query::Error(e),
        }
    }

    /// Y方向の Extrude (絶対座標による引き延ばし) 演算を適用する
    pub fn extrude_y<T: Into<u8>, P>(self, z: T, start_y: u32, end_y: u32, _policy: P) -> Self
    where
        P: MergePolicy<S::Value> + Send + Sync,
    {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        match ZoomLevel::new(z.into()) {
            Ok(zl) => Query::Unary(
                Box::new(ExtrudeY::<P>::new(zl, start_y, end_y)),
                Box::new(self),
            ),
            Err(e) => Query::Error(e),
        }
    }

    /// F方向の Extrude (絶対座標による引き延ばし) 演算を適用する
    pub fn extrude_f<T: Into<u8>, P>(self, z: T, start_f: i32, end_f: i32, _policy: P) -> Self
    where
        P: MergePolicy<S::Value> + Send + Sync,
    {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        match ZoomLevel::new(z.into()) {
            Ok(zl) => Query::Unary(
                Box::new(ExtrudeF::<P>::new(zl, start_f, end_f)),
                Box::new(self),
            ),
            Err(e) => Query::Error(e),
        }
    }

    /// F, X, Y方向の Extrude (絶対座標による一括引き延ばし) 演算を適用する
    #[allow(clippy::too_many_arguments)]
    pub fn extrude_fxy<T: Into<u8>, P>(
        self,
        z: T,
        start_f: i32,
        end_f: i32,
        start_x: u32,
        end_x: u32,
        start_y: u32,
        end_y: u32,
        _policy: P,
    ) -> Self
    where
        P: MergePolicy<S::Value> + Send + Sync,
    {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        match ZoomLevel::new(z.into()) {
            Ok(zl) => {
                let op = ExtrudeFXY::<P>::new(zl, start_f, end_f, start_x, end_x, start_y, end_y);
                Query::Unary(Box::new(op), Box::new(self))
            }
            Err(e) => Query::Error(e),
        }
    }
}
