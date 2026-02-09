/// 2Dベクトル型（投影後の座標用）
#[derive(Debug, Clone, Copy)]
pub struct Vec2 {
    x: f64,
    y: f64,
}

impl Vec2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }
}
