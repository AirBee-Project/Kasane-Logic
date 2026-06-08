use crate::{ConflictPolicy, FlexId, SpatialIdCollection};

/// Level 系演算子の共通パラメータ。ズーム `z` の絶対座標範囲 `[lo, hi]` を指定する。
///
/// `stretch` が相対量で占有を継ぎ足して起伏を保存するのに対し、`level` は対象次元の占有を
/// 絶対範囲 `[lo, hi]` へ置き換えるため、その次元の起伏（凹凸）は平坦化される。
/// 範囲外への占有は削られ、足りない区間は埋められる。値が重なったときの解決方針を
/// [`conflict`](Self::conflict) で指定する（値が1種の `Set`/`bool` では結果は変わらない）。
///
/// 座標型 `C` は次元により異なる（F は `i32`、X/Y は `u32`）。
pub struct LevelParam<C, V> {
    /// 範囲を表すズームレベル。
    pub z: u8,
    /// 範囲の一方の端（順序は次元の規約に従う）。
    pub lo: C,
    /// 範囲のもう一方の端。
    pub hi: C,
    /// 占有が重なったときの解決方針。
    pub conflict: ConflictPolicy<V>,
}

/// 高さ（F）方向の絶対範囲へ揃える
pub mod level_f;

/// 東西（X）方向の絶対範囲へ揃える
pub mod level_x;

/// 南北（Y）方向の絶対範囲へ揃える
pub mod level_y;

/// Level 系演算子をメソッドとして呼び出す拡張トレイト
pub mod ops;

#[cfg(test)]
mod tests;

/// 範囲へ揃えたセル `cell` を `result` へ、衝突方針 `conflict` に従って書き込む。
///
/// `Overwrite` は素の挿入と等価なので問い合わせを省く。それ以外は既存値を読み出して
/// 畳み込む。混在ズームで複数の既存セルが重なる場合は先頭の値を既存値として扱う。
pub(super) fn insert_leveled<O>(
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
