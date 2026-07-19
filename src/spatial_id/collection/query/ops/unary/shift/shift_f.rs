use alloc::boxed::Box;

use crate::{
    Error, FlexTreeCore, ZoomLevel, spatial_id::collection::flex_tree::core::SafeValue,
    spatial_id::collection::query::traits::UnaryOperator,
};

/// 作業木全体を高さ（F）方向へ、ズームレベル `z` のセル `f` 個分だけ平行移動する単項演算。
///
/// F-shift は単射だが `f<0 ↔ f≥0` の符号跨ぎがあり、構造シフトの上下ルート独立処理では扱えない。
/// よって per-cell（[`FlexTreeCore::map_rebuild`]、union で符号ごとに正しく振り分け）で組み直す。
/// 移動後が範囲外になる場合は [`Error`] を返す。
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

impl<V: SafeValue> UnaryOperator<V> for ShiftF {
    fn run(
        &self,
        target: &mut FlexTreeCore<V>,
    ) -> Result<(), Box<dyn core::error::Error + 'static>> {
        let z = self.z.get();
        let index = self.f;
        if index == 0 {
            return Ok(());
        }

        *target = target.map_rebuild(|id, value| {
            let value = value.clone();
            Ok(id
                .shift_f(z, index)?
                .map(move |moved| (moved, value.clone())))
        })?;
        Ok(())
    }
}
