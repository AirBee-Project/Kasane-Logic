/// 2次元のベクトル型
#[derive(Debug, Clone, Copy)]
pub struct Vec2 {
    x: f64,
    y: f64,
}

impl Vec2 {
    /// 新しい Vec2 を作成
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// x 成分を返す
    pub fn x(&self) -> f64 {
        self.x
    }

    /// y 成分を返す
    pub fn y(&self) -> f64 {
        self.y
    }

    /// ベクトルの加算
    pub fn add(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x + other.x(), self.y + other.y())
    }

    /// ベクトルの減算
    pub fn sub(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x - other.x(), self.y - other.y())
    }

    /// スカラー倍
    pub fn mul(self, scalar: f64) -> Vec2 {
        Vec2::new(self.x * scalar, self.y * scalar)
    }

    /// 内積（dot product）
    pub fn dot(self, other: Vec2) -> f64 {
        self.x * other.x() + self.y * other.y()
    }

    /// ベクトルの長さの二乗
    pub fn length_sq(self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    /// ベクトルの長さ
    pub fn length(self) -> f64 {
        self.length_sq().sqrt()
    }

    /// ベクトルの正規化（長さ 1 にする）
    /// 長さがほぼ 0 の場合は None を返す
    pub fn normalize(self) -> Option<Vec2> {
        let len = self.length();
        if len < 1.0e-15 {
            None
        } else {
            Some(self.mul(1.0 / len))
        }
    }

    /// 他ベクトルとの距離
    pub fn distance(self, other: Vec2) -> f64 {
        self.sub(other).length()
    }

    /// 他ベクトルとの距離の二乗
    pub fn distance_sq(self, other: Vec2) -> f64 {
        self.sub(other).length_sq()
    }
}
