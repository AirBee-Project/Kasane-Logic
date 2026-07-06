use crate::spatial_id::collection::query::Query;
use crate::{CellValue, ConflictPolicy, Error, FlexId, SpatialIdCollection, UnaryOperator};
use alloc::vec;
use alloc::vec::Vec;

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

impl<C: SpatialIdCollection> Query<C>
where
    C::Value: 'static,
{
    pub fn shift_f(self, z: u8, index: i32) -> Self {
        Query::Unary(
            crate::spatial_id::collection::query::ops::unary::UnaryOp::Shift(ShiftParam::f(
                z, index,
            )),
            alloc::boxed::Box::new(self),
        )
    }

    pub fn shift_x(self, z: u8, index: i32) -> Self {
        Query::Unary(
            crate::spatial_id::collection::query::ops::unary::UnaryOp::Shift(ShiftParam::x(
                z, index,
            )),
            alloc::boxed::Box::new(self),
        )
    }

    pub fn shift_y(self, z: u8, index: i32) -> Self {
        Query::Unary(
            crate::spatial_id::collection::query::ops::unary::UnaryOp::Shift(ShiftParam::y(
                z, index,
            )),
            alloc::boxed::Box::new(self),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::SpatialIdCollection;
    use crate::spatial_id::zoom_level::ZoomLevel;
    use crate::{FlexId, SingleId, SpatialIdSet, SpatialIdTable};

    fn table_with(z: u8, f: i32, x: u32, y: u32) -> SpatialIdTable<bool> {
        let mut table = SpatialIdTable::new();
        table.insert(SingleId::new(z, f, x, y).unwrap(), true);
        table
    }

    /// 指定したセル（z, f, x, y）に値が存在するか。
    fn present(table: &SpatialIdTable<bool>, z: u8, f: i32, x: u32, y: u32) -> bool {
        let cell = SingleId::new(z, f, x, y).unwrap();
        table.get(&cell).next().is_some()
    }

    // ---- F方向（鉛直・巡回なし） ----

    #[test]
    fn shift_f_up_moves_cell() {
        let table = table_with(25, 0, 100, 100);
        let result = table.clone().query().shift_f(25, 3).run().unwrap();

        assert!(!present(&result, 25, 0, 100, 100)); // 元の位置は空く
        assert!(present(&result, 25, 3, 100, 100)); // +3 へ移動
    }

    #[test]
    fn shift_f_down_moves_cell() {
        let table = table_with(25, 10, 100, 100);
        let result = table.clone().query().shift_f(25, -4).run().unwrap();

        assert!(!present(&result, 25, 10, 100, 100));
        assert!(present(&result, 25, 6, 100, 100));
    }

    #[test]
    fn shift_f_out_of_range_is_error() {
        // z=25 の最上セルをさらに上へ動かすと範囲外。
        let table = table_with(25, ZoomLevel::new(25_u8).unwrap().f_max(), 100, 100);
        assert!(table.clone().query().shift_f(25, 1).run().is_err());
    }

    // ---- X方向（東西・巡回する） ----

    #[test]
    fn shift_x_moves_cell() {
        let table = table_with(25, 0, 100, 100);
        let result = table.clone().query().shift_x(25, 5).run().unwrap();

        assert!(!present(&result, 25, 0, 100, 100));
        assert!(present(&result, 25, 0, 105, 100));
    }

    #[test]
    fn shift_x_wraps_around_seam() {
        // z=2 の最東セル x=3 を +1 すると、経度の境界を越えて x=0 へ巡回する。
        let table = table_with(2, 0, 3, 0);
        let result = table.clone().query().shift_x(2, 1).run().unwrap();

        assert!(!present(&result, 2, 0, 3, 0));
        assert!(present(&result, 2, 0, 0, 0));
    }

    #[test]
    fn shift_x_full_circle_returns_to_origin() {
        // z=2 では一周が4セル。+4 で元の位置へ戻る。
        let table = table_with(2, 0, 1, 0);
        let result = table.clone().query().shift_x(2, 4).run().unwrap();

        assert!(present(&result, 2, 0, 1, 0));
    }

    #[test]
    fn shift_x_splits_when_crossing_seam() {
        // X方向のみ粗い（z1）セルは z2 換算で2セル幅。+1 でちょうど境界をまたぎ、
        // RangeIdの巡回表現と同様に z2 の2セル（x=3 と x=0）へ分割される。
        let mut table = SpatialIdTable::new();
        let id = FlexId::new(2, 0, 1, 1, 2, 0).unwrap(); // x だけ z1 / index1（= z2 の 2,3）
        table.insert(id, true);

        let result = table.clone().query().shift_x(2, 1).run().unwrap();

        assert!(present(&result, 2, 0, 3, 0)); // 元の 3 が残り
        assert!(present(&result, 2, 0, 0, 0)); // 4 が巡回して 0 へ
        assert!(!present(&result, 2, 0, 2, 0)); // 元の 2 は空く
    }

    // ---- Y方向（南北・巡回なし） ----

    #[test]
    fn shift_y_moves_cell() {
        let table = table_with(25, 0, 100, 100);
        let result = table.clone().query().shift_y(25, -3).run().unwrap();

        assert!(!present(&result, 25, 0, 100, 100));
        assert!(present(&result, 25, 0, 100, 97));
    }

    #[test]
    fn shift_y_below_zero_is_error() {
        // y=0 を下方向へ動かすと範囲外。
        let table = table_with(25, 0, 100, 0);
        assert!(table.clone().query().shift_y(25, -1).run().is_err());
    }

    #[test]
    fn shift_y_above_max_is_error() {
        // z=2 の最北セル y=3 を上へ動かすと範囲外。
        let table = table_with(2, 0, 0, 3);
        assert!(table.clone().query().shift_y(2, 1).run().is_err());
    }

    // ---- Set でも同じ演算が使える（総称化の確認） ----

    #[test]
    fn shift_works_on_set() {
        let mut set = SpatialIdSet::new();
        set.insert(SingleId::new(25, 0, 100, 100).unwrap());

        let result = set.clone().query().shift_f(25, 3).run().unwrap();

        let moved = SingleId::new(25, 3, 100, 100).unwrap();
        let original = SingleId::new(25, 0, 100, 100).unwrap();
        assert!(result.get(&moved).next().is_some());
        assert!(result.get(&original).next().is_none());
    }
}
