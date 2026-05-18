use crate::{Coordinate, Ecef};
use std::ops::{Add, Sub};
pub mod impls;

#[derive(Debug, Clone, Copy, PartialEq)]
/// 3次元空間におけるベクトルを表す型。
///
/// X, Y, Z の各成分によって定義される。
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    /// 新しい `Vec3` を生成する。
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
    /// 内積（ドット積）を計算する。
    pub fn dot(&self, other: &Vec3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// 他の `Vec3` との外積（クロス積）を計算する。
    pub fn cross(&self, other: &Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// ベクトルの絶対値（ノルム・長さ）を計算する。
    pub fn norm(&self) -> f64 {
        libm::sqrt(self.norm_squared())
    }

    /// ベクトルの絶対値の2乗（ノルムの2乗）を計算する。
    pub fn norm_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// 同じ向きの単位ベクトル（ノルムが1のベクトル）を計算して返す。
    /// ゼロベクトルの場合は計算できないため `None` を返す。
    pub fn normalize(&self) -> Option<Vec3> {
        let n = self.norm();
        if n == 0.0 {
            None
        } else {
            Some(Vec3 {
                x: self.x / n,
                y: self.y / n,
                z: self.z / n,
            })
        }
    }

    /// ベクトルを定数倍（スカラー倍）した新しいベクトルを返す。
    pub fn scale(&self, scalar: f64) -> Vec3 {
        Vec3 {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
    /// 引数となるベクトルに垂直な平面の直交基底を返す。始点は原点。
    pub fn create_orthonormal_basis(&self) -> [Self; 2] {
        if self.x == 0.0 && self.y == 0.0 {
            [Self::new(1.0, 0.0, 0.0), Self::new(0.0, 1.0, 0.0)]
        } else {
            [
                Self::new(-self.y, self.x, 0.0).normalize().unwrap(),
                Self::new(
                    -self.z * self.x,
                    -self.z * self.y,
                    self.x * self.x + self.y * self.y,
                )
                .normalize()
                .unwrap(),
            ]
        }
    }
}

impl From<[f64; 3]> for Vec3 {
    fn from(arr: [f64; 3]) -> Self {
        Vec3 {
            x: arr[0],
            y: arr[1],
            z: arr[2],
        }
    }
}

impl From<Vec3> for [f64; 3] {
    fn from(vec: Vec3) -> Self {
        [vec.x, vec.y, vec.z]
    }
}

impl From<Ecef> for Vec3 {
    fn from(ecef: Ecef) -> Self {
        Vec3 {
            x: ecef.x(),
            y: ecef.y(),
            z: ecef.z(),
        }
    }
}

impl From<Vec3> for Ecef {
    fn from(vec: Vec3) -> Self {
        Ecef::new(vec.x, vec.y, vec.z)
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Vec3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Vec3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl From<Coordinate> for Vec3 {
    fn from(coord: Coordinate) -> Self {
        let ecef: Ecef = coord.into();
        ecef.into()
    }
}
