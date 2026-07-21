use crate::spatial_id::collection::query::execution::group_commutative::types::CommutativityInfo;
use crate::{
    Error, ZoomLevel,
    spatial_id::collection::query::traits::{UnaryOperator, WorkingTree},
};

/// 作業木全体を高さ（F）方向へ、ズームレベル `z` のインデックス値 `f` 個分だけ平行移動する単項演算。
pub struct ShiftF {
    z: ZoomLevel,
    f: i32,
}

impl ShiftF {
    /// ズーム `z` のインデックス値 `f` 個分の高さ移動を表す演算子を作る。
    pub fn new<T: Into<u8>>(z: T, f: i32) -> Result<Self, Error> {
        let z = ZoomLevel::new(z.into())?;
        Ok(Self { z, f })
    }
}

impl<W: WorkingTree + 'static> UnaryOperator<W> for ShiftF {
    fn commutativity_info(&self) -> CommutativityInfo {
        CommutativityInfo::separable_injective()
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

    fn fmt_op(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "shift_f(z={}, f={})", self.z.get(), self.f)
    }
}
