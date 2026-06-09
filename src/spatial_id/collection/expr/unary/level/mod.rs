#[allow(unused_imports)]
use alloc::boxed::Box;
#[allow(unused_imports)]
use alloc::rc::Rc;
#[allow(unused_imports)]
use alloc::string::{String, ToString};
#[allow(unused_imports)]
use alloc::vec::Vec;

use crate::{ConflictPolicy, FlexId, SpatialIdCollection};

pub struct LevelParam<C, V> {
    /// 範囲を表すズームレベル。
    pub z: u8,
    /// 範囲の一方の端。
    pub lo: C,
    /// 範囲のもう一方の端。
    pub hi: C,
    /// 占有が重なったときの解決方針。
    pub conflict: ConflictPolicy<V>,
}

/// F方向の絶対範囲へ揃える
pub mod level_f;

/// X方向の絶対範囲へ揃える
pub mod level_x;

/// Y方向の絶対範囲へ揃える
pub mod level_y;

pub mod ops;

#[cfg(test)]
mod tests;

/// 範囲へ揃えたセル `cell` を `result` へ、衝突方針 `conflict` に従って書き込む。
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
