use crate::SpatialIdCollection;
use alloc::boxed::Box;

pub trait BinaryOperator<T: SpatialIdCollection>: Send + Sync {
    /// コレクションに対する二項演算子の定義
    fn run(
        &self,
        target_a: &mut T,
        target_b: &T,
    ) -> Result<(), Box<dyn core::error::Error + 'static>>;
}

/// 空間IDコレクションに対して単項演算を行うTrait。
/// パラメーターは各演算子の構造体フィールドとして保持する（例: [`ShiftX`](super::ops::unary::shift::shift_x::ShiftX)）。
pub trait UnaryOperator<T: SpatialIdCollection>: Send + Sync {
    /// コレクションに対する単項演算の定義
    fn run(&self, target: &mut T) -> Result<(), Box<dyn core::error::Error + 'static>>;
}
