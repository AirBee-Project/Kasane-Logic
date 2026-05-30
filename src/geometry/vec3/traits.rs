use std::ops::{Add, Sub};

/// 3次元空間におけるベクトルを表すトレイト。
///
/// A, B, C の各成分によって定義される。
pub trait Vec3: Sized + Add<Output = Self> + Sub<Output = Self> {
    /// 新しい `Vec3` を生成する。
    fn new(a: f64, b: f64, c: f64) -> Self;

    /// A成分（XまたはF成分）を返す。
    fn a(&self) -> f64;

    /// B成分（YまたはX成分）を返す。
    fn b(&self) -> f64;

    /// C成分（ZまたはY成分）を返す。
    fn c(&self) -> f64;

    /// 内積（ドット積）を計算する。
    fn dot(&self, other: &Self) -> f64 {
        self.a() * other.a() + self.b() * other.b() + self.c() * other.c()
    }

    /// 他のベクトルとの外積（クロス積）を計算する。
    fn cross(&self, other: &Self) -> Self {
        Self::new(
            self.b() * other.c() - self.c() * other.b(),
            self.c() * other.a() - self.a() * other.c(),
            self.a() * other.b() - self.b() * other.a(),
        )
    }

    /// ベクトルの絶対値（ノルム・長さ）を計算する。
    fn norm(&self) -> f64 {
        libm::sqrt(self.norm_squared())
    }

    /// ベクトルの絶対値の2乗（ノルムの2乗）を計算する。
    fn norm_squared(&self) -> f64 {
        self.a() * self.a() + self.b() * self.b() + self.c() * self.c()
    }

    /// 同じ向きの単位ベクトル（ノルムが1のベクトル）を計算して返す。
    /// ゼロベクトルの場合は計算できないため `None` を返す。
    fn normalize(&self) -> Option<Self> {
        let n = self.norm();
        if n == 0.0 {
            None
        } else {
            Some(Self::new(self.a() / n, self.b() / n, self.c() / n))
        }
    }

    /// ベクトルを定数倍（スカラー倍）した新しいベクトルを返す。
    fn scale(&self, scalar: f64) -> Self {
        Self::new(self.a() * scalar, self.b() * scalar, self.c() * scalar)
    }

    /// 引数となるベクトルに垂直な平面の直交基底を返す。始点は原点。
    fn create_orthonormal_basis(&self) -> [Self; 2] {
        if self.a() == 0.0 && self.b() == 0.0 {
            [Self::new(1.0, 0.0, 0.0), Self::new(0.0, 1.0, 0.0)]
        } else {
            [
                Self::new(-self.b(), self.a(), 0.0).normalize().unwrap(),
                Self::new(
                    -self.c() * self.a(),
                    -self.c() * self.b(),
                    self.a() * self.a() + self.b() * self.b(),
                )
                .normalize()
                .unwrap(),
            ]
        }
    }
}
