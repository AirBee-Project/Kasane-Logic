use alloc::boxed::Box;

use crate::{
    Error, FlexTreeCore, ZoomLevel, spatial_id::collection::flex_tree::core::SafeValue,
    spatial_id::collection::query::traits::UnaryOperator,
};

/// 作業木全体を東西（X）方向へ、ズームレベル `z` のセル `x` 個分だけ平行移動する単項演算。
///
/// X-shift は空間的に単射（各セル → 移動先が重ならない）なので union で組み直す
/// （[`FlexTreeCore::map_rebuild`]）。X 方向は経度 ±180 度で巡回する。
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

impl<V: SafeValue> UnaryOperator<V> for ShiftX {
    fn run(
        &self,
        target: &mut FlexTreeCore<V>,
    ) -> Result<(), Box<dyn core::error::Error + 'static>> {
        let z = self.z.get();
        let index = self.x;
        if index == 0 {
            return Ok(());
        }

        *target = target.map_rebuild(|id, value| {
            let value = value.clone();
            Ok(id.shift_x(z, index)?.map(move |m| (m, value.clone())))
        })?;
        Ok(())
    }
}
