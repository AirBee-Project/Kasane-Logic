use std::ops::{Add, Sub};

use crate::{Error, FractionalId, Vec3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3FractionalId {
    pub a: f64,
    pub b: f64,
    pub c: f64,
}

impl Vec3FractionalId {
    pub fn fractional_id(&self, z: u8) -> Result<FractionalId, Error> {
        let id = FractionalId::new(z, self.a, self.b, self.c)?;
        Ok(id)
    }
}

impl Add for Vec3FractionalId {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            a: self.a + other.a,
            b: self.b + other.b,
            c: self.c + other.c,
        }
    }
}

impl Sub for Vec3FractionalId {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            a: self.a - other.a,
            b: self.b - other.b,
            c: self.c - other.c,
        }
    }
}

impl Vec3 for Vec3FractionalId {
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

impl From<FractionalId> for Vec3FractionalId {
    fn from(id: FractionalId) -> Self {
        Vec3FractionalId {
            a: id.f(),
            b: id.x(),
            c: id.y(),
        }
    }
}
