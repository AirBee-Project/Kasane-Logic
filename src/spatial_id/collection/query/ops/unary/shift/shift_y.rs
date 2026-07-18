use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::{
    Error, FlexId, SpatialIdCollection, ZoomLevel,
    spatial_id::collection::query::traits::UnaryOperator,
};

/// コレクション全体を南北（Y）方向へ、ズームレベル `z` のセル `y` 個分だけ平行移動する単項演算。
pub struct ShiftY {
    z: ZoomLevel,
    y: i32,
}

impl ShiftY {
    /// ズーム `z` のセル `y` 個分の南北移動を表す演算子を作る。
    pub fn new<T: Into<u8>>(z: T, y: i32) -> Result<Self, Error> {
        let z = ZoomLevel::new(z.into())?;
        Ok(Self { z, y })
    }
}

impl<T: SpatialIdCollection> UnaryOperator<T> for ShiftY {
    fn run(&self, target: &mut T) -> Result<(), Box<dyn core::error::Error + 'static>> {
        let z = self.z.get();
        let index = self.y;

        if index == 0 {
            return Ok(());
        }

        let shifted = target.map_rebuild(|id, value| {
            let value = value.clone();
            let cells: Vec<(FlexId, T::Value)> = id
                .shift_y(z, index)?
                .map(move |moved| (moved, value.clone()))
                .collect();
            Ok(cells)
        })?;

        *target = shifted;
        Ok(())
    }
}
