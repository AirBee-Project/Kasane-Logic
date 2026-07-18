use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::{
    Error, FlexId, SpatialIdCollection, ZoomLevel,
    spatial_id::collection::query::traits::UnaryOperator,
};

/// コレクション全体を東西（X）方向へ、ズームレベル `z` のセル `x` 個分だけ平行移動する単項演算。
///
/// 各格納要素を [`FlexId::shift_x`] で移動し、コレクションを作り直す。移動は要素ごとに独立なので
/// [`SpatialIdCollection::map_rebuild`] を介して写像・再構築の双方が並列化され、FlexTree の
/// マルチコア実装をそのまま活かす。X 方向は経度 ±180 度で巡回する。
pub struct ShiftX {
    z: ZoomLevel,
    x: i32,
}

impl ShiftX {
    /// ズーム `z` のセル `x` 個分の東西移動を表す演算子を作る。
    pub fn new<T: Into<u8>>(z: T, x: i32) -> Result<Self, Error> {
        let z = ZoomLevel::new(z.into())?;
        Ok(Self { z, x })
    }
}

impl<T: SpatialIdCollection> UnaryOperator<T> for ShiftX {
    fn run(&self, target: &mut T) -> Result<(), Box<dyn core::error::Error + 'static>> {
        let z = self.z.get();
        let index = self.x;

        if index == 0 {
            return Ok(());
        }

        let shifted = target.map_rebuild(|id, value| {
            let value = value.clone();
            let cells: Vec<(FlexId, T::Value)> = id
                .shift_x(z, index)?
                .map(move |moved| (moved, value.clone()))
                .collect();
            Ok(cells)
        })?;

        *target = shifted;
        Ok(())
    }
}
