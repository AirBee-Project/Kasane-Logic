/// Shift 系演算子の共通パラメータ。ズーム `z` のセル `index` 個分だけ移動する。
pub struct ShiftParam {
    /// 移動量の単位となるズームレベル。
    pub z: u8,
    /// 移動量のインデックス値。
    pub index: i32,
}

/// 高さ（F）方向への移動
pub mod shift_f;

/// 東西（X）方向への移動
pub mod shift_x;

/// 南北（Y）方向への移動
pub mod shift_y;

/// Shift 系演算子をメソッドとして呼び出す拡張トレイト
pub mod ops;

#[cfg(test)]
mod tests;
