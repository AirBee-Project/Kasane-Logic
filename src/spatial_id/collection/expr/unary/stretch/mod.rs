use crate::{ConflictPolicy, FlexId, SpatialIdCollection};

pub struct StretchParam<V> {
    /// 引き延ばし量の単位となるズームレベル。
    pub z: u8,
    /// 引き延ばし量のインデックス値。
    pub index: i32,
    /// 重なったときの解決方針。
    pub conflict: ConflictPolicy<V>,
}

/// F方向への引き延ばし
pub mod stretch_f;

/// X方向への引き延ばし
pub mod stretch_x;

/// Y方向への引き延ばし
pub mod stretch_y;

pub mod ops;

#[cfg(test)]
mod tests;

/// 伸長したセル `cell` を `result` へ、衝突方針 `conflict` に従って書き込む。
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
