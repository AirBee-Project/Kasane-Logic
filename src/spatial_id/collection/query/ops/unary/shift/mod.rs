/// X方向への移動演算子
pub mod shift_x;

/// Y方向への移動演算子
pub mod shift_y;

/// F方向への移動演算子
pub mod shift_f;

/// 3次元統合移動演算子
pub mod shift_fxy;

/// FlexId単体に対する実装
pub mod primitive;

/// Query型への実装
pub mod query;

#[cfg(test)]
mod test;
