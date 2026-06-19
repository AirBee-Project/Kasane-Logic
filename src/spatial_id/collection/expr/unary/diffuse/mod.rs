use core::ops::Sub;

use alloc::vec::Vec;

use crate::{ConflictPolicy, Error, FlexId, SpatialIdCollection, SpatialIdError};

/// Diffuse 系演算子の共通パラメータ。
///
/// 値を持つセル（発生源）から、ある次元の `±` 両方向へ最大 `distance` ステップだけ
/// 値を波及させる。1ステップ進むごとに値から `decay` を引き、引けなくなった時点
/// （`value <= decay`）で打ち切る。発生源自身の値はそのまま残る。
pub struct DiffuseParam<V> {
    /// 1ステップの移動量の単位となるズームレベル。
    pub z: u8,
    /// 各方向へ波及させる最大ステップ数。
    pub distance: u32,
    /// 1ステップごとに値から引く減衰量。
    pub decay: V,
    /// 波及先が既存の値と重なったときの解決方針。
    pub conflict: ConflictPolicy<V>,
}

/// F方向への波及
pub mod diffuse_f;

/// X方向への波及
pub mod diffuse_x;

/// Y方向への波及
pub mod diffuse_y;

pub mod ops;

#[cfg(test)]
mod tests;

/// 波及したセル `cell` を `result` へ、衝突方針 `conflict` に従って書き込む。
pub(super) fn insert_diffused<O>(
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

/// F・Y方向の境界超過エラーか。X方向は巡回するため該当しない。
fn is_out_of_range(error: &Error) -> bool {
    matches!(
        error,
        Error::SpatialId(SpatialIdError::FOutOfRange { .. } | SpatialIdError::YOutOfRange { .. })
    )
}

/// 1つの次元に沿った波及演算の汎用ドライバ。
///
/// `step` は「対象セルをズーム `z` のセル `index` 個分動かした位置」を返す関数で、
/// 次元ごとに `shift_f` / `shift_x` / `shift_y` を渡す。`clip` が `true` のとき、
/// 境界超過（[`is_out_of_range`]）はエラーにせずその向きのみ打ち切る。
pub(super) fn diffuse_along<S, O, F>(
    a: &S,
    param: DiffuseParam<O::Value>,
    clip: bool,
    step: F,
) -> Result<O, Error>
where
    S: SpatialIdCollection,
    O: SpatialIdCollection<Value = S::Value>,
    O::Value: Sub<Output = O::Value>,
    F: Fn(&FlexId, u8, i32) -> Result<Vec<FlexId>, Error>,
{
    let DiffuseParam {
        z,
        distance,
        decay,
        conflict,
    } = param;

    let mut result = O::empty();
    for (flex_id, value) in a.scan() {
        // 発生源の値はそのまま残す。
        insert_diffused(&mut result, flex_id.clone(), value.clone(), &conflict);

        let mut current = value;
        for offset in 1..=distance {
            // これ以上引くと 0 以下になるなら打ち切る。
            if current <= decay {
                break;
            }
            current = current - decay.clone();

            for dir in [1i32, -1i32] {
                let index = dir * offset as i32;
                match step(&flex_id, z, index) {
                    Ok(cells) => {
                        for cell in cells {
                            insert_diffused(&mut result, cell, current.clone(), &conflict);
                        }
                    }
                    Err(error) if clip && is_out_of_range(&error) => continue,
                    Err(error) => return Err(error),
                }
            }
        }
    }

    Ok(result)
}
