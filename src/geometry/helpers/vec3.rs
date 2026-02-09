use crate::Coordinate;

/// 3Dベクトル型（法線計算・投影用）
#[derive(Debug, Clone, Copy)]
pub struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn dot(self, other: Vec3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn sub(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x - other.x(), self.y - other.y(), self.z - other.z())
    }

    /// ベクトルの長さの二乗を返す
    ///
    /// # 返り値
    /// |self|^2 = x^2 + y^2 + z^2
    pub fn length_sq(self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn from_coord(coord: &Coordinate) -> Self {
        Self::new(
            coord.as_longitude(),
            coord.as_latitude(),
            coord.as_altitude(),
        )
    }

    pub fn cross(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn length(self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalize(self) -> Option<Vec3> {
        let len = self.length();
        if len < 1.0e-15 {
            None
        } else {
            Some(Vec3 {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
            })
        }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn z(&self) -> f64 {
        self.z
    }
}
