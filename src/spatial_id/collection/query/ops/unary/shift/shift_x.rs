use super::shift_fxy::ShiftFXY;
use crate::spatial_id::collection::query::execution::group_commutative::types::CommutativityInfo;
use crate::{
    Error, ZoomLevel,
    spatial_id::collection::query::traits::{
        UnaryOperator, WorkingTree, try_merge_via_accumulator,
    },
};
use alloc::boxed::Box;

/// 作業木全体を東西（X）方向へ、ズームレベル `z` のセル `x` 個分だけ平行移動する単項演算。
///
/// X-shift は空間的に単射（各セル → 移動先が重ならない）なので union で組み直す
/// （[`WorkingTree::map_rebuild`]）。X 方向は経度 ±180 度で巡回する。
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

    pub(crate) fn z(&self) -> ZoomLevel {
        self.z
    }

    pub(crate) fn x(&self) -> i32 {
        self.x
    }
}

impl<W: WorkingTree + 'static> UnaryOperator<W> for ShiftX {
    fn validate(&self) -> Result<(), Error> {
        let zl = ZoomLevel::new(self.z.get())?;
        zl.check_x(self.x.unsigned_abs())?;
        Ok(())
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }

    fn run(&self, target: &mut W) -> Result<(), Error> {
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

    fn commutativity_info(&self) -> CommutativityInfo {
        CommutativityInfo::separable_injective()
    }

    /// 同じズームレベルの `ShiftX`/`ShiftY`/`ShiftF`/`ShiftFXY` は、オフセットを加算した1つの
    /// `ShiftFXY` に統合できる（平行移動の合成は軸ごとの加算そのもの）。統合先の型が
    /// `ShiftFXY` に変わる点に注意（`ShiftFXY` は他軸オフセット0なら `ShiftX` と等価に動作する）。
    fn try_merge(&self, other: &dyn UnaryOperator<W>) -> Option<Box<dyn UnaryOperator<W>>> {
        try_merge_via_accumulator::<W, ShiftFXY>(self, other)
    }
}
