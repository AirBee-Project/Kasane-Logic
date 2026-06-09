#[allow(unused_imports)]
use alloc::boxed::Box;
#[allow(unused_imports)]
use alloc::rc::Rc;
#[allow(unused_imports)]
use alloc::string::{String, ToString};
#[allow(unused_imports)]
use alloc::vec::Vec;

use crate::{ConflictPolicy, Error, SpatialIdCollection, UnaryOperator};

use super::LevelParam;
use super::level_f::FLevel;
use super::level_x::XLevel;
use super::level_y::YLevel;

pub trait LevelOps: SpatialIdCollection {
    fn level_f(&self, z: u8, lo: i32, hi: i32) -> Result<Self, Error> {
        self.level_f_with(z, lo, hi, ConflictPolicy::Overwrite)
    }

    fn level_x(&self, z: u8, lo: u32, hi: u32) -> Result<Self, Error> {
        self.level_x_with(z, lo, hi, ConflictPolicy::Overwrite)
    }

    fn level_y(&self, z: u8, lo: u32, hi: u32) -> Result<Self, Error> {
        self.level_y_with(z, lo, hi, ConflictPolicy::Overwrite)
    }

    fn level_f_with(
        &self,
        z: u8,
        lo: i32,
        hi: i32,
        conflict: ConflictPolicy<Self::Value>,
    ) -> Result<Self, Error> {
        FLevel::execution::<Self, Self>(
            self,
            LevelParam {
                z,
                lo,
                hi,
                conflict,
            },
        )
    }

    fn level_x_with(
        &self,
        z: u8,
        lo: u32,
        hi: u32,
        conflict: ConflictPolicy<Self::Value>,
    ) -> Result<Self, Error> {
        XLevel::execution::<Self, Self>(
            self,
            LevelParam {
                z,
                lo,
                hi,
                conflict,
            },
        )
    }

    fn level_y_with(
        &self,
        z: u8,
        lo: u32,
        hi: u32,
        conflict: ConflictPolicy<Self::Value>,
    ) -> Result<Self, Error> {
        YLevel::execution::<Self, Self>(
            self,
            LevelParam {
                z,
                lo,
                hi,
                conflict,
            },
        )
    }
}

impl<C> LevelOps for C where C: SpatialIdCollection {}
