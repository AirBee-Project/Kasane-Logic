use super::shift_fxy::ShiftFXY;
use crate::spatial_id::collection::query::execution::group_commutative::types::CommutativityInfo;
use crate::{
    Error, ZoomLevel,
    spatial_id::collection::query::traits::{
        UnaryOperator, WorkingTree, try_merge_via_accumulator,
    },
};
use alloc::boxed::Box;

/// 作業木全体を高さ（F）方向へ、ズームレベル `z` のセル `f` 個分だけ平行移動する単項演算。
///
/// F-shift は単射だが `f<0 ↔ f≥0` の符号跨ぎがあり、構造シフトの上下ルート独立処理では扱えない。
/// よって per-cell（[`WorkingTree::map_rebuild`]、union で符号ごとに正しく振り分け）で組み直す。
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

    pub(crate) fn z(&self) -> ZoomLevel {
        self.z
    }

    pub(crate) fn f(&self) -> i32 {
        self.f
    }
}

impl<W: WorkingTree + 'static> UnaryOperator<W> for ShiftF {
    fn commutativity_info(&self) -> CommutativityInfo {
        CommutativityInfo::separable_injective()
    }

    /// 同じズームレベルの `ShiftX`/`ShiftY`/`ShiftF`/`ShiftFXY` は、オフセットを加算した1つの
    /// `ShiftFXY` に統合できる（平行移動の合成は軸ごとの加算そのもの）。
    fn try_merge(&self, other: &dyn UnaryOperator<W>) -> Option<Box<dyn UnaryOperator<W>>> {
        try_merge_via_accumulator::<W, ShiftFXY>(self, other)
    }

    fn validate(&self) -> Result<(), Error> {
        let zl = ZoomLevel::new(self.z.get())?;
        zl.check_f(self.f)?;
        Ok(())
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }

    fn run(&self, target: &mut W) -> Result<(), Error> {
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
