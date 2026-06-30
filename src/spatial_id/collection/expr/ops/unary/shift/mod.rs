use alloc::vec;
use alloc::vec::Vec;

use crate::{CellValue, ConflictPolicy, Error, FlexId, SpatialIdCollection, UnaryOperator};

/// 集合演算をメソッドとして呼び出す拡張トレイト
pub mod ops;

#[cfg(test)]
mod tests;

/// 1 軸ぶんの移動量。ズーム `z` のセル `index` 個分だけ動かす。
pub struct ShiftAmount {
    /// 移動量の単位となるズームレベル。
    pub z: u8,
    /// 移動量のインデックス値。
    pub index: i32,
}

/// Shift 演算子のパラメータ。F / X / Y 各軸の移動量を保持する。
///
/// 各軸の移動は互いに独立なので、軸が衝突しない（同じ軸を両方が持たない）限り
/// 複数の Shift を 1 回の走査へ融合できる。存在しない軸は `None`。
pub struct ShiftParam {
    /// 高さ（F）方向の移動。
    pub f: Option<ShiftAmount>,
    /// 東西（X）方向の移動。
    pub x: Option<ShiftAmount>,
    /// 南北（Y）方向の移動。
    pub y: Option<ShiftAmount>,
}

impl ShiftParam {
    /// 高さ（F）方向の単一軸移動を作る。
    pub fn f(z: u8, index: i32) -> Self {
        Self {
            f: Some(ShiftAmount { z, index }),
            x: None,
            y: None,
        }
    }

    /// 東西（X）方向の単一軸移動を作る。
    pub fn x(z: u8, index: i32) -> Self {
        Self {
            f: None,
            x: Some(ShiftAmount { z, index }),
            y: None,
        }
    }

    /// 南北（Y）方向の単一軸移動を作る。
    pub fn y(z: u8, index: i32) -> Self {
        Self {
            f: None,
            x: None,
            y: Some(ShiftAmount { z, index }),
        }
    }

    /// すべての軸が移動なし（恒等変換）かどうか。
    pub fn is_identity(&self) -> bool {
        let is_zero = |a: &Option<ShiftAmount>| a.as_ref().is_none_or(|s| s.index == 0);
        is_zero(&self.f) && is_zero(&self.x) && is_zero(&self.y)
    }
}

/// 空間IDコレクションを、指定した各軸へ平行移動する単項演算。
///
/// X 方向は地球を周回するため巡回し、Y / F 方向は範囲外への移動がエラーになる。
/// 各軸は独立なので、複数軸を 1 度の走査でまとめて適用できる。
pub struct Shift;

impl<A: CellValue> UnaryOperator<A> for Shift {
    type CustomParameter = ShiftParam;
    type ResultValue = A;

    fn execution<S, O>(a: S, param: Self::CustomParameter) -> Result<O, Error>
    where
        S: SpatialIdCollection<Value = A>,
        O: SpatialIdCollection<Value = A>,
    {
        let cells: Vec<(FlexId, A)> = a.scan().collect();
        let shifted = super::map_cells(cells, |id| apply(id.clone(), &param))?;

        Ok(O::from_cells(shifted, &ConflictPolicy::Overwrite))
    }

    fn is_identity(param: &Self::CustomParameter) -> bool {
        param.is_identity()
    }
}

/// 1 つのセルへ、存在する軸の移動を X → Y → F の順に適用する。
/// 各軸は独立なので適用順は最終結果に影響しない。
fn apply(flex_id: FlexId, param: &ShiftParam) -> Result<Vec<FlexId>, Error> {
    let ids = vec![flex_id];
    let ids = apply_axis(ids, &param.x, |id, z, i| Ok(id.shift_x(z, i)?.collect()))?;
    let ids = apply_axis(ids, &param.y, |id, z, i| Ok(id.shift_y(z, i)?.collect()))?;
    let ids = apply_axis(ids, &param.f, |id, z, i| Ok(id.shift_f(z, i)?.collect()))?;
    Ok(ids)
}

/// `amount` が `Some` のとき、各セルへ 1 軸の移動を適用して展開する。
/// `None` のときは入力をそのまま返す。
fn apply_axis<F>(
    ids: Vec<FlexId>,
    amount: &Option<ShiftAmount>,
    shift: F,
) -> Result<Vec<FlexId>, Error>
where
    F: Fn(&FlexId, u8, i32) -> Result<Vec<FlexId>, Error>,
{
    let Some(ShiftAmount { z, index }) = amount else {
        return Ok(ids);
    };

    let mut out = Vec::new();
    for id in ids {
        out.extend(shift(&id, *z, *index)?);
    }
    Ok(out)
}
