use crate::spatial_id::collection::query::execution::group_commutative::types::CommutativityInfo;
use crate::{
    Error, ZoomLevel,
    spatial_id::collection::query::traits::{UnaryOperator, WorkingTree},
};

/// 作業木全体を東西（X）方向へ、ズームレベル `z` のインデックス値 `x` 個分だけ平行移動する単項演算。
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

    fn inverse_bounds(&self, bounds: crate::RangeId) -> alloc::vec::Vec<crate::RangeId> {
        let target_z = bounds.z();
        let z = self.z.get();
        let max_z = z.max(target_z);
        let shift_z = max_z - z;
        let scale_t = max_z - target_z;
        
        let delta = (self.x as i64) * (1i64 << shift_z);
        
        let x_min_max_z = (bounds.x()[0] as i64) * (1i64 << scale_t);
        let x_max_max_z = ((bounds.x()[1] as i64) + 1) * (1i64 << scale_t) - 1;
        
        let new_min_max_z = x_min_max_z - delta;
        let new_max_max_z = x_max_max_z - delta;
        
        let max_len = 1i64 << max_z;
        let new_min_max_z_wrapped = new_min_max_z.rem_euclid(max_len);
        let new_max_max_z_wrapped = new_max_max_z.rem_euclid(max_len);
        
        let mut x_ranges = alloc::vec::Vec::new();
        if new_min_max_z_wrapped <= new_max_max_z_wrapped {
            x_ranges.push((new_min_max_z_wrapped, new_max_max_z_wrapped));
        } else {
            x_ranges.push((new_min_max_z_wrapped, max_len - 1));
            x_ranges.push((0, new_max_max_z_wrapped));
        }
        
        let mut res = alloc::vec::Vec::new();
        for (min_max_z, max_max_z) in x_ranges {
            let mut new_bounds = bounds.clone();
            let new_min_target = (min_max_z >> scale_t) as u32;
            let new_max_target = (max_max_z >> scale_t) as u32;
            new_bounds.set_x([new_min_target, new_max_target]).unwrap();
            res.push(new_bounds);
        }
        res
    }

    fn commutativity_info(&self) -> CommutativityInfo {
        CommutativityInfo::separable_injective()
    }

    fn fmt_op(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "shift_x(z={}, x={})", self.z.get(), self.x)
    }
}
