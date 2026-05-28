use std::ops::{Add, Sub};

pub trait Vec3: Sized + Add<Output = Self> + Sub<Output = Self> {
    fn new(a: f64, b: f64, c: f64) -> Self;
    fn a(&self) -> f64;
    fn b(&self) -> f64;
    fn c(&self) -> f64;

    fn dot(&self, other: &Self) -> f64 {
        self.a() * other.a() + self.b() * other.b() + self.c() * other.c()
    }

    fn cross(&self, other: &Self) -> Self {
        Self::new(
            self.b() * other.c() - self.c() * other.b(),
            self.c() * other.a() - self.a() * other.c(),
            self.a() * other.b() - self.b() * other.a(),
        )
    }

    fn norm(&self) -> f64 {
        libm::sqrt(self.norm_squared())
    }

    fn norm_squared(&self) -> f64 {
        self.a() * self.a() + self.b() * self.b() + self.c() * self.c()
    }

    fn normalize(&self) -> Option<Self> {
        let n = self.norm();
        if n == 0.0 {
            None
        } else {
            Some(Self::new(self.a() / n, self.b() / n, self.c() / n))
        }
    }

    fn scale(&self, scalar: f64) -> Self {
        Self::new(self.a() * scalar, self.b() * scalar, self.c() * scalar)
    }

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
