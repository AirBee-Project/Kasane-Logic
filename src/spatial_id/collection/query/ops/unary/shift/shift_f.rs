use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::{
    Error, FlexId, SpatialIdCollection, ZoomLevel,
    spatial_id::collection::query::traits::UnaryOperator,
};

/// コレクション全体を高さ（F）方向へ、ズームレベル `z` のセル `f` 個分だけ平行移動する単項演算。
pub struct ShiftF {
    z: ZoomLevel,
    f: i32,
}

impl ShiftF {
    /// ズーム `z` のセル `f` 個分の高さ移動を表す演算子を作る。
    pub fn new<T: Into<u8>>(z: T, f: i32) -> Result<Self, Error> {
        let z = ZoomLevel::new(z.into())?;
        Ok(Self { z, f })
    }
}

impl<T: SpatialIdCollection> UnaryOperator<T> for ShiftF {
    fn run(&self, target: &mut T) -> Result<(), Box<dyn std::error::Error + 'static>> {
        let z = self.z.get();
        let index = self.f;

        if index == 0 {
            return Ok(());
        }

        let shifted = target.map_rebuild(|id, value| {
            let value = value.clone();
            let cells: Vec<(FlexId, T::Value)> = id
                .shift_f(z, index)?
                .map(move |moved| (moved, value.clone()))
                .collect();
            Ok(cells)
        })?;

        *target = shifted;
        Ok(())
    }
}
