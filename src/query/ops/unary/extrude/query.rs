use super::{Extrude, ExtrudeAxis};
use crate::query::Node;
use crate::query::SafeValue;
use crate::{
    ZoomLevel,
    query::{Query, merge_policy::MergePolicy},
};

impl<V: SafeValue + 'static> Query<V> {
    fn extrude<T: Into<u8>, P>(self, z: T, axis: ExtrudeAxis) -> Self
    where
        P: MergePolicy<V> + Send + Sync,
    {
        if matches!(self.0, Node::Error(_)) {
            return self;
        }
        match ZoomLevel::new(z.into()) {
            Ok(zl) => self.wrap_unary(Extrude::<P>::new(zl, axis)),
            Err(e) => Query(Node::Error(e)),
        }
    }

    /// X方向の Extrude (絶対座標による引き延ばし) 演算を適用する
    pub fn extrude_x<T: Into<u8>, P>(self, z: T, start_x: u32, end_x: u32, _policy: P) -> Self
    where
        P: MergePolicy<V> + Send + Sync,
    {
        self.extrude::<T, P>(z, ExtrudeAxis::X(start_x, end_x))
    }

    /// Y方向の Extrude (絶対座標による引き延ばし) 演算を適用する
    pub fn extrude_y<T: Into<u8>, P>(self, z: T, start_y: u32, end_y: u32, _policy: P) -> Self
    where
        P: MergePolicy<V> + Send + Sync,
    {
        self.extrude::<T, P>(z, ExtrudeAxis::Y(start_y, end_y))
    }

    /// F方向の Extrude (絶対座標による引き延ばし) 演算を適用する
    pub fn extrude_f<T: Into<u8>, P>(self, z: T, start_f: i32, end_f: i32, _policy: P) -> Self
    where
        P: MergePolicy<V> + Send + Sync,
    {
        self.extrude::<T, P>(z, ExtrudeAxis::F(start_f, end_f))
    }
}
