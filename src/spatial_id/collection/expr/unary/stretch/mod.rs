use alloc::vec;
use alloc::vec::Vec;

use crate::{CellValue, ConflictPolicy, Error, FlexId, SpatialIdCollection, UnaryOperator};

/// 集合演算をメソッドとして呼び出す拡張トレイト
pub mod ops;

#[cfg(test)]
mod tests;

/// 1 軸ぶんの引き延ばし量。ズーム `z` のセル `index` 個分だけ拡張する。
pub struct StretchAmount {
    /// 引き延ばし量の単位となるズームレベル。
    pub z: u8,
    /// 引き延ばし量のインデックス値。
    pub index: i32,
}

/// Stretch 演算子のパラメータ。F / X / Y 各軸の引き延ばし量と、重なりの解決方針を保持する。
/// 存在しない軸は `None`。
pub struct StretchParam<V> {
    /// 高さ（F）方向の引き延ばし。
    pub f: Option<StretchAmount>,
    /// 東西（X）方向の引き延ばし。
    pub x: Option<StretchAmount>,
    /// 南北（Y）方向の引き延ばし。
    pub y: Option<StretchAmount>,
    /// 引き延ばしで重なったときの解決方針。
    pub conflict: ConflictPolicy<V>,
}

impl<V> StretchParam<V> {
    /// 高さ（F）方向の単一軸引き延ばしを作る。
    pub fn f(z: u8, index: i32, conflict: ConflictPolicy<V>) -> Self {
        Self {
            f: Some(StretchAmount { z, index }),
            x: None,
            y: None,
            conflict,
        }
    }

    /// 東西（X）方向の単一軸引き延ばしを作る。
    pub fn x(z: u8, index: i32, conflict: ConflictPolicy<V>) -> Self {
        Self {
            f: None,
            x: Some(StretchAmount { z, index }),
            y: None,
            conflict,
        }
    }

    /// 南北（Y）方向の単一軸引き延ばしを作る。
    pub fn y(z: u8, index: i32, conflict: ConflictPolicy<V>) -> Self {
        Self {
            f: None,
            x: None,
            y: Some(StretchAmount { z, index }),
            conflict,
        }
    }

    /// すべての軸が拡張なし（恒等変換）かどうか。
    pub fn is_identity(&self) -> bool {
        let is_zero = |a: &Option<StretchAmount>| a.as_ref().is_none_or(|s| s.index == 0);
        is_zero(&self.f) && is_zero(&self.x) && is_zero(&self.y)
    }
}

/// 空間IDコレクションを、指定した各軸へ引き延ばす（元のセルを残したまま拡張する）単項演算。
///
/// X 方向は地球を周回するため巡回し、Y / F 方向は範囲外への拡張がエラーになる。
pub struct Stretch;

impl<A: CellValue> UnaryOperator<A> for Stretch {
    type CustomParameter = StretchParam<A>;
    type ResultValue = A;

    fn execution<S, O>(a: &S, param: Self::CustomParameter) -> Result<O, Error>
    where
        S: SpatialIdCollection<Value = A>,
        O: SpatialIdCollection<Value = A>,
    {
        let cells: Vec<(FlexId, A)> = a.scan().collect();
        let stretched = super::map_cells(cells, |id| expand(id.clone(), &param))?;

        Ok(O::from_cells(stretched, &param.conflict))
    }

    fn is_identity(param: &Self::CustomParameter) -> bool {
        param.is_identity()
    }
}

/// 1 つのセルへ、存在する軸の引き延ばしを X → Y → F の順に適用して展開する。
fn expand<V>(flex_id: FlexId, param: &StretchParam<V>) -> Result<Vec<FlexId>, Error> {
    let ids = vec![flex_id];
    let ids = apply_axis(ids, &param.x, |id, z, i| Ok(id.stretch_x(z, i)?.collect()))?;
    let ids = apply_axis(ids, &param.y, |id, z, i| Ok(id.stretch_y(z, i)?.collect()))?;
    let ids = apply_axis(ids, &param.f, |id, z, i| Ok(id.stretch_f(z, i)?.collect()))?;
    Ok(ids)
}

/// `amount` が `Some` のとき、各セルへ 1 軸の引き延ばしを適用して展開する。
fn apply_axis<F>(
    ids: Vec<FlexId>,
    amount: &Option<StretchAmount>,
    stretch: F,
) -> Result<Vec<FlexId>, Error>
where
    F: Fn(&FlexId, u8, i32) -> Result<Vec<FlexId>, Error> + Send + Sync,
{
    let Some(StretchAmount { z, index }) = amount else {
        return Ok(ids);
    };

    #[cfg(feature = "rayon")]
    {
        use rayon::prelude::*;
        ids.into_par_iter()
            .map(|id| stretch(&id, *z, *index))
            .collect::<Result<Vec<Vec<_>>, Error>>()
            .map(|grouped| grouped.into_iter().flatten().collect())
    }

    #[cfg(not(feature = "rayon"))]
    {
        let mut out = Vec::new();
        for id in ids {
            out.extend(stretch(&id, *z, *index)?);
        }
        Ok(out)
    }
}
