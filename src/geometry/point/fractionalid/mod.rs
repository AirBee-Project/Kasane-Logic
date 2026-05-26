pub mod impls;
use crate::{
    error::{Error, SpatialIdError},
    spatial_id::constants::{F_MAX, F_MIN, MAX_ZOOM_LEVEL, XY_MAX},
};
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct FractionalId {
    z: u8,
    f: f64,
    x: f64,
    y: f64,
}
impl FractionalId {
    pub fn new(z: u8, f: f64, x: f64, y: f64) -> Result<Self, Error> {
        if z > MAX_ZOOM_LEVEL as u8 {
            return Err(SpatialIdError::ZOutOfRange { z }.into());
        }

        let f_min = F_MIN[z as usize] as f64;
        let f_max = F_MAX[z as usize] as f64;
        let xy_max = XY_MAX[z as usize] as f64;

        if f < f_min || f > f_max {
            return Err(SpatialIdError::FOutOfRange {
                f: libm::floor(f) as i32,
                z,
            }
            .into());
        }
        if x > xy_max {
            return Err(SpatialIdError::XOutOfRange {
                x: libm::floor(x) as u32,
                z,
            }
            .into());
        }
        if y > xy_max {
            return Err(SpatialIdError::YOutOfRange {
                y: libm::floor(y) as u32,
                z,
            }
            .into());
        }

        Ok(FractionalId { z, f, x, y })
    }
    /// * `z` が有効なズームレベル（0–[MAX_ZOOM_LEVEL]）であること
    /// * `f` が与えられた `z` に応じて `F_MIN[z]..=F_MAX[z]` の範囲内であること
    /// * `x` および `y` が `0..=XY_MAX[z]` の範囲内であること
    ///
    /// これらが保証されない場合、パニック・不正メモリアクセス・未定義動作を引き起こす可能性がある。
    ///
    /// # Safety
    ///
    pub unsafe fn new_unchecked(z: u8, f: f64, x: f64, y: f64) -> Self {
        FractionalId { z, f, x, y }
    }
}
