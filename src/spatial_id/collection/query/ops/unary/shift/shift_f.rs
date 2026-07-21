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

    fn inverse_bounds(&self, mut bounds: crate::RangeId) -> alloc::vec::Vec<crate::RangeId> {
        let target_z = bounds.z();
        let z = self.z.get();
        let max_z = z.max(target_z);
        let shift_z = max_z - z;
        let scale_t = max_z - target_z;

        let delta = (self.f as i64) * (1i64 << shift_z);

        let f_min_max_z = (bounds.f()[0] as i64) * (1i64 << scale_t);
        let f_max_max_z = ((bounds.f()[1] as i64) + 1) * (1i64 << scale_t) - 1;

        let max_z_obj = ZoomLevel::new(max_z).unwrap();
        let min_f = max_z_obj.f_min() as i64;
        let max_f = max_z_obj.f_max() as i64;

        let new_min_max_z = (f_min_max_z - delta).clamp(min_f, max_f);
        let new_max_max_z = (f_max_max_z - delta).clamp(min_f, max_f);

        if new_min_max_z <= new_max_max_z {
            let new_min_target = (new_min_max_z >> scale_t) as i32;
            let new_max_target = (new_max_max_z >> scale_t) as i32;
            bounds.set_f([new_min_target, new_max_target]).unwrap();
            alloc::vec![bounds]
        } else {
            alloc::vec![]
        }
    }

    fn fmt_op(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "shift_f(z={}, f={})", self.z.get(), self.f)
    }
}
