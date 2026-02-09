use crate::Coordinate;

/// 3次元のベクトル型
#[derive(Debug, Clone, Copy)]
pub struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Vec3 {
    /// 新しい Vec3 を作成
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Coordinate から Vec3 を作成
    pub fn from_coord(coord: &Coordinate) -> Self {
        Self::new(
            coord.as_longitude(),
            coord.as_latitude(),
            coord.as_altitude(),
        )
    }

    /// ベクトルの x 成分を返す
    pub fn x(&self) -> f64 {
        self.x
    }

    /// ベクトルの y 成分を返す
    pub fn y(&self) -> f64 {
        self.y
    }

    /// ベクトルの z 成分を返す
    pub fn z(&self) -> f64 {
        self.z
    }

    /// ベクトルの加算
    pub fn add(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x + other.x(), self.y + other.y(), self.z + other.z())
    }

    /// ベクトルの減算
    pub fn sub(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x - other.x(), self.y - other.y(), self.z - other.z())
    }

    /// ベクトルのスカラー倍
    pub fn mul(self, scalar: f64) -> Vec3 {
        Vec3::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }

    /// 内積（dot product）
    pub fn dot(self, other: Vec3) -> f64 {
        self.x * other.x() + self.y * other.y() + self.z * other.z()
    }

    /// 外積（cross product）
    pub fn cross(self, other: Vec3) -> Vec3 {
        Vec3::new(
            self.y * other.z() - self.z * other.y(),
            self.z * other.x() - self.x * other.z(),
            self.x * other.y() - self.y * other.x(),
        )
    }

    /// ベクトルの長さの二乗
    pub fn length_sq(self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// ベクトルの長さ
    pub fn length(self) -> f64 {
        self.length_sq().sqrt()
    }

    /// ベクトルの正規化（長さ 1 にする）
    /// 長さがほぼ 0 の場合は None を返す
    pub fn normalize(self) -> Option<Vec3> {
        let len = self.length();
        if len < 1.0e-15 {
            None
        } else {
            Some(self.mul(1.0 / len))
        }
    }

    /// 他ベクトルとの距離
    pub fn distance(self, other: Vec3) -> f64 {
        self.sub(other).length()
    }

    /// 他ベクトルとの距離の二乗
    pub fn distance_sq(self, other: Vec3) -> f64 {
        self.sub(other).length_sq()
    }
}
