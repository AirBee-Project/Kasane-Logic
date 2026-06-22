use alloc::vec;
use alloc::vec::Vec;

use crate::{CellValue, ConflictPolicy, Error, FlexId, SpatialIdCollection, UnaryOperator};

/// 集合演算をメソッドとして呼び出す拡張トレイト
pub mod ops;

#[cfg(test)]
mod tests;

/// 1 軸ぶんの「揃える絶対範囲」。ズーム `z` における `[lo, hi]`。
///
/// 座標型 `C` は軸で異なる（F は符号付き `i32`、X / Y は `u32`）。
pub struct LevelAmount<C> {
    /// 範囲を表すズームレベル。
    pub z: u8,
    /// 範囲の一方の端。
    pub lo: C,
    /// 範囲のもう一方の端。
    pub hi: C,
}

/// Level 演算子のパラメータ。F / X / Y 各軸の絶対範囲と、重なりの解決方針を保持する。
/// 存在しない軸は `None`。
pub struct LevelParam<V> {
    /// 高さ（F）方向の範囲（符号付き）。
    pub f: Option<LevelAmount<i32>>,
    /// 東西（X）方向の範囲。
    pub x: Option<LevelAmount<u32>>,
    /// 南北（Y）方向の範囲。
    pub y: Option<LevelAmount<u32>>,
    /// 占有が重なったときの解決方針。
    pub conflict: ConflictPolicy<V>,
}

impl<V> LevelParam<V> {
    /// 高さ（F）方向の単一軸 Level を作る。
    pub fn f(z: u8, lo: i32, hi: i32, conflict: ConflictPolicy<V>) -> Self {
        Self {
            f: Some(LevelAmount { z, lo, hi }),
            x: None,
            y: None,
            conflict,
        }
    }

    /// 東西（X）方向の単一軸 Level を作る。
    pub fn x(z: u8, lo: u32, hi: u32, conflict: ConflictPolicy<V>) -> Self {
        Self {
            f: None,
            x: Some(LevelAmount { z, lo, hi }),
            y: None,
            conflict,
        }
    }

    /// 南北（Y）方向の単一軸 Level を作る。
    pub fn y(z: u8, lo: u32, hi: u32, conflict: ConflictPolicy<V>) -> Self {
        Self {
            f: None,
            x: None,
            y: Some(LevelAmount { z, lo, hi }),
            conflict,
        }
    }
}

/// 特定の次元の占有を絶対座標範囲へ揃える（起伏を平坦化する）単項演算。
///
/// X 方向は巡回するため `lo` から東向きに `hi` まで、Y / F は範囲外がエラーになる。
pub struct Level;

impl<A: CellValue> UnaryOperator<A> for Level {
    type CustomParameter = LevelParam<A>;
    type ResultValue = A;

    fn execution<S, O>(a: &S, param: Self::CustomParameter) -> Result<O, Error>
    where
        S: SpatialIdCollection<Value = A>,
        O: SpatialIdCollection<Value = A>,
    {
        let cells: Vec<(FlexId, A)> = a.scan().collect();
        let leveled = super::map_cells(cells, |id| expand(id.clone(), &param))?;

        // 重なり解決は `result` を参照するため、元の順序のまま逐次 insert する。
        let mut result = O::empty();
        for (id, value) in leveled {
            insert_leveled(&mut result, id, value, &param.conflict);
        }
        Ok(result)
    }

    fn is_identity(_param: &Self::CustomParameter) -> bool {
        false
    }
}

/// 1 つのセルへ、存在する軸の Level を X → Y → F の順に適用して展開する。
fn expand<V>(flex_id: FlexId, param: &LevelParam<V>) -> Result<Vec<FlexId>, Error> {
    let ids = vec![flex_id];
    let ids = apply_axis(ids, &param.x, |id, z, lo, hi| {
        Ok(id.level_x(z, lo, hi)?.collect())
    })?;
    let ids = apply_axis(ids, &param.y, |id, z, lo, hi| {
        Ok(id.level_y(z, lo, hi)?.collect())
    })?;
    let ids = apply_axis(ids, &param.f, |id, z, lo, hi| {
        Ok(id.level_f(z, lo, hi)?.collect())
    })?;
    Ok(ids)
}

/// `amount` が `Some` のとき、各セルへ 1 軸の Level を適用して展開する。
fn apply_axis<C, F>(
    ids: Vec<FlexId>,
    amount: &Option<LevelAmount<C>>,
    level: F,
) -> Result<Vec<FlexId>, Error>
where
    C: Copy + Send + Sync,
    F: Fn(&FlexId, u8, C, C) -> Result<Vec<FlexId>, Error> + Send + Sync,
{
    let Some(LevelAmount { z, lo, hi }) = amount else {
        return Ok(ids);
    };

    #[cfg(feature = "rayon")]
    {
        use rayon::prelude::*;
        ids.into_par_iter()
            .map(|id| level(&id, *z, *lo, *hi))
            .collect::<Result<Vec<Vec<_>>, Error>>()
            .map(|grouped| grouped.into_iter().flatten().collect())
    }

    #[cfg(not(feature = "rayon"))]
    {
        let mut out = Vec::new();
        for id in ids {
            out.extend(level(&id, *z, *lo, *hi)?);
        }
        Ok(out)
    }
}

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
