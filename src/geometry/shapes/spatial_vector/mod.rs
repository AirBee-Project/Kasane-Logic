pub mod impls;


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
}
