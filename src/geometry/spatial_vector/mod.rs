use crate::{Coordinate, Ecef};
use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy, PartialEq)]
/// 3次元空間におけるベクトルを表す型。
///
/// X, Y, Z の各成分によって定義される。
pub struct SpatialVector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl SpatialVector {
    /// 新しい `SpatialVector` を生成する。
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
    /// 内積（ドット積）を計算する。
    pub fn dot(&self, other: &SpatialVector) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// 他の `SpatialVector` との外積（クロス積）を計算する。
    pub fn cross(&self, other: &SpatialVector) -> SpatialVector {
        SpatialVector {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// ベクトルの絶対値（ノルム・長さ）を計算する。
    pub fn norm(&self) -> f64 {
        self.norm_squared().sqrt()
    }

    /// ベクトルの絶対値の2乗（ノルムの2乗）を計算する。
    pub fn norm_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// 同じ向きの単位ベクトル（ノルムが1のベクトル）を計算して返す。
    /// ゼロベクトルの場合は計算できないため `None` を返す。
    pub fn normalize(&self) -> Option<SpatialVector> {
        let n = self.norm();
        if n == 0.0 {
            None
        } else {
            Some(SpatialVector {
                x: self.x / n,
                y: self.y / n,
                z: self.z / n,
            })
        }
    }

    /// ベクトルを定数倍（スカラー倍）した新しいベクトルを返す。
    pub fn scale(&self, scalar: f64) -> SpatialVector {
        SpatialVector {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
    /// 引数となるベクトルに垂直な平面の直交基底を返す。始点は原点。
    pub fn create_orthonormal_basis(&self) -> [Self; 2] {
        if self.x == 0.0 && self.y == 0.0 {
            return [Self::new(1.0, 0.0, 0.0), Self::new(0.0, 1.0, 0.0)];
        } else {
            return [
                Self::new(-self.y, self.x, 0.0).normalize().unwrap(),
                Self::new(
                    -self.z * self.x,
                    -self.z * self.y,
                    self.x * self.x + self.y * self.y,
                )
                .normalize()
                .unwrap(),
            ];
        }
    }
}

impl From<[f64; 3]> for SpatialVector {
    fn from(arr: [f64; 3]) -> Self {
        SpatialVector {
            x: arr[0],
            y: arr[1],
            z: arr[2],
        }
    }
}

impl From<SpatialVector> for [f64; 3] {
    fn from(vec: SpatialVector) -> Self {
        [vec.x, vec.y, vec.z]
    }
}

impl From<Ecef> for SpatialVector {
    fn from(ecef: Ecef) -> Self {
        SpatialVector {
            x: ecef.x(),
            y: ecef.y(),
            z: ecef.z(),
        }
    }
}

impl From<SpatialVector> for Ecef {
    fn from(vec: SpatialVector) -> Self {
        Ecef::new(vec.x, vec.y, vec.z)
    }
}

impl Add for SpatialVector {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        SpatialVector {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub for SpatialVector {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        SpatialVector {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl From<Coordinate> for SpatialVector {
    fn from(coord: Coordinate) -> Self {
        let ecef: Ecef = coord.into();
        ecef.into()
    }
}
