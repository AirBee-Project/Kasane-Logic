use crate::{ConflictPolicy, FlexId, SpatialIdCollection};

/// Stretch 系演算子の共通パラメータ。ズーム `z` のセル `index` 個分だけ引き延ばす。
///
/// 伸長は単射ではないため、別々のセルが同じセルへ重なりうる。その衝突をどう解決するかを
/// [`conflict`](Self::conflict) で指定する（値が1種の `Set`/`bool` では結果は変わらない）。
pub struct StretchParam<V> {
    /// 引き延ばし量の単位となるズームレベル。
    pub z: u8,
    /// 引き延ばし量のインデックス値（符号で方向、`0` で恒等）。
    pub index: i32,
    /// 伸長で値が重なったときの解決方針。
    pub conflict: ConflictPolicy<V>,
}

/// 高さ（F）方向への引き延ばし
pub mod stretch_f;

/// 東西（X）方向への引き延ばし
pub mod stretch_x;

/// 南北（Y）方向への引き延ばし
pub mod stretch_y;

/// Stretch 系演算子をメソッドとして呼び出す拡張トレイト
pub mod ops;

#[cfg(test)]
mod tests;

/// 伸長したセル `cell` を `result` へ、衝突方針 `conflict` に従って書き込む。
///
/// `Overwrite` は素の挿入と等価なので問い合わせを省く。それ以外は既存値を読み出して
/// 畳み込む。混在ズームで複数の既存セルが重なる場合は先頭の値を既存値として扱う。
pub(super) fn insert_stretched<O>(
    result: &mut O,
    cell: FlexId,
    value: O::Value,
    conflict: &ConflictPolicy<O::Value>,
) where
    O: SpatialIdCollection,
{
    let resolved = if let ConflictPolicy::Overwrite = conflict {
        value
    } else {
        let current = result.query(&cell).next().map(|(_, v)| v);
        conflict.resolve(current, value)
    };
    result.insert(cell, resolved);
}
