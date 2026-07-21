use crate::spatial_id::collection::query::execution::group_commutative::types::CommutativityInfo;
use crate::{
    Error, ZoomLevel,
    spatial_id::collection::query::traits::{UnaryOperator, WorkingTree},
};

/// 作業木全体を南北（Y）方向へ、ズームレベル `z` のインデックス値 `y` 個分だけ平行移動する単項演算。
pub struct ShiftY {
    z: ZoomLevel,
    y: i32,
}

impl ShiftY {
    /// ズーム `z` のインデックス値 `y` 個分の南北移動を表す演算子を作る。
    pub fn new<T: Into<u8>>(z: T, y: i32) -> Result<Self, Error> {
        let z = ZoomLevel::new(z.into())?;
        Ok(Self { z, y })
    }
}

impl<W: WorkingTree + 'static> UnaryOperator<W> for ShiftY {
    fn validate(&self) -> Result<(), Error> {
        let zl = ZoomLevel::new(self.z.get())?;
        zl.check_y(self.y.unsigned_abs())?;
        Ok(())
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }

    fn run(&self, target: &mut W) -> Result<(), Error> {
        let z = self.z.get();
        let index = self.y;
        if index == 0 {
            return Ok(());
        }

        *target = target.map_rebuild(|id, value| {
            let value = value.clone();
            Ok(id.shift_y(z, index)?.map(move |m| (m, value.clone())))
        })?;
        Ok(())
    }

    fn commutativity_info(&self) -> CommutativityInfo {
        CommutativityInfo::separable_injective()
    }

    fn fmt_op(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "shift_y(z={}, y={})", self.z.get(), self.y)
    }
}
