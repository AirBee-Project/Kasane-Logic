use core::ops::{Add, Sub};

use crate::geometry::vec3::traits::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
/// 地心直交座標(Ecef)をベクトルとして扱ったもの。
/// a は X成分、b は Y成分、c は Z成分を表す。
/// 座標系を混同しないため、Vec3FractionalId とは区別する。
pub struct Vec3Ecef {
    pub a: f64,
    pub b: f64,
    pub c: f64,
}

impl Add for Vec3Ecef {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            a: self.a + other.a,
            b: self.b + other.b,
            c: self.c + other.c,
        }
    }
}

impl Sub for Vec3Ecef {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            a: self.a - other.a,
            b: self.b - other.b,
            c: self.c - other.c,
        }
    }
}

impl Vec3 for Vec3Ecef {
    fn new(a: f64, b: f64, c: f64) -> Self {
        Self { a, b, c }
    }
    fn a(&self) -> f64 {
        self.a
    }
    fn b(&self) -> f64 {
        self.b
    }
    fn c(&self) -> f64 {
        self.c
    }
}

impl From<crate::Ecef> for Vec3Ecef {
    fn from(ecef: crate::Ecef) -> Self {
        Vec3Ecef {
            a: ecef.x(),
            b: ecef.y(),
            c: ecef.z(),
        }
    }
}

impl From<Vec3Ecef> for crate::Ecef {
    fn from(vec: Vec3Ecef) -> Self {
        crate::Ecef::new(vec.a, vec.b, vec.c)
    }
}

impl From<crate::Coordinate> for Vec3Ecef {
    fn from(coord: crate::Coordinate) -> Self {
        let ecef: crate::Ecef = coord.into();
        ecef.into()
    }
}
