/// Shift 系演算子の共通パラメータ。ズーム `z` のセル `index` 個分だけ移動する。
pub struct ShiftParam {
    /// 移動量の単位となるズームレベル。
    pub z: u8,
    /// 移動量のインデックス値。
    pub index: i32,
}

/// F方向への移動
pub mod shift_f;

/// X方向への移動
pub mod shift_x;

/// Y方向への移動
pub mod shift_y;

pub mod ops;

#[cfg(test)]
mod tests;
