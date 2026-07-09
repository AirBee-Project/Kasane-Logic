use crate::spatial_id::collection::query::Query;
use crate::{CellValue, ConflictPolicy, Error, FlexId, SpatialIdCollection, UnaryOperator};
use alloc::vec;
use alloc::vec::Vec;

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

    fn execution<S, O>(a: S, param: Self::CustomParameter) -> Result<O, Error>
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
            .try_fold(Vec::new, |mut acc, chunk| {
                acc.extend(chunk?);
                Ok(acc)
            })
            .try_reduce(Vec::new, |mut a, b| {
                a.extend(b);
                Ok(a)
            })
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

impl<C: SpatialIdCollection> Query<C>
where
    C::Value: 'static,
{
    pub fn stretch_f(self, z: u8, index: i32) -> Self {
        self.stretch_f_with(z, index, ConflictPolicy::Overwrite)
    }

    pub fn stretch_x(self, z: u8, index: i32) -> Self {
        self.stretch_x_with(z, index, ConflictPolicy::Overwrite)
    }

    pub fn stretch_y(self, z: u8, index: i32) -> Self {
        self.stretch_y_with(z, index, ConflictPolicy::Overwrite)
    }

    pub fn stretch_f_with(self, z: u8, index: i32, conflict: ConflictPolicy<C::Value>) -> Self {
        Query::Unary(
            crate::spatial_id::collection::query::ops::unary::UnaryOp::Stretch(StretchParam::f(
                z, index, conflict,
            )),
            alloc::boxed::Box::new(self),
        )
    }

    pub fn stretch_x_with(self, z: u8, index: i32, conflict: ConflictPolicy<C::Value>) -> Self {
        Query::Unary(
            crate::spatial_id::collection::query::ops::unary::UnaryOp::Stretch(StretchParam::x(
                z, index, conflict,
            )),
            alloc::boxed::Box::new(self),
        )
    }

    pub fn stretch_y_with(self, z: u8, index: i32, conflict: ConflictPolicy<C::Value>) -> Self {
        Query::Unary(
            crate::spatial_id::collection::query::ops::unary::UnaryOp::Stretch(StretchParam::y(
                z, index, conflict,
            )),
            alloc::boxed::Box::new(self),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::SpatialIdCollection;
    use crate::spatial_id::zoom_level::ZoomLevel;
    use crate::{ConflictPolicy, FlexId, SingleId, SpatialIdSet, SpatialIdTable};

    fn table_with(z: u8, f: i32, x: u32, y: u32) -> SpatialIdTable<bool> {
        let mut table = SpatialIdTable::new();
        table.insert(SingleId::new(z, f, x, y).unwrap(), true);
        table
    }

    fn present(table: &SpatialIdTable<bool>, z: u8, f: i32, x: u32, y: u32) -> bool {
        let cell = SingleId::new(z, f, x, y).unwrap();
        table.get(&cell).next().is_some()
    }

    // ---- F方向 ----

    #[test]
    fn stretch_f_keeps_original_and_fills_up() {
        // f=0 を +3 引き延ばす → 元の 0 と 1..=3 が埋まる（shift と違い 0 は残る）。
        let table = table_with(25, 0, 100, 100);
        let result = table.clone().query().stretch_f(25, 3).run().unwrap();

        for f in 0..=3 {
            assert!(present(&result, 25, f, 100, 100), "f={f} should be filled");
        }
        assert!(!present(&result, 25, 4, 100, 100));
        assert!(!present(&result, 25, -1, 100, 100));
    }

    #[test]
    fn stretch_f_negative_fills_down() {
        // f=10 を -2 引き延ばす → 8..=10 が埋まる。
        let table = table_with(25, 10, 100, 100);
        let result = table.clone().query().stretch_f(25, -2).run().unwrap();

        for f in 8..=10 {
            assert!(present(&result, 25, f, 100, 100), "f={f} should be filled");
        }
        assert!(!present(&result, 25, 7, 100, 100));
        assert!(!present(&result, 25, 11, 100, 100));
    }

    #[test]
    fn stretch_f_zero_is_identity() {
        let table = table_with(25, 4, 100, 100);
        let result = table.clone().query().stretch_f(25, 0).run().unwrap();

        assert!(present(&result, 25, 4, 100, 100));
        assert!(!present(&result, 25, 3, 100, 100));
        assert!(!present(&result, 25, 5, 100, 100));
    }

    #[test]
    fn stretch_f_out_of_range_is_error() {
        let table = table_with(25, ZoomLevel::new(25_u8).unwrap().f_max(), 100, 100);
        assert!(table.clone().query().stretch_f(25, 1).run().is_err());
    }

    // ---- X方向（巡回） ----

    #[test]
    fn stretch_x_fills_range() {
        let table = table_with(25, 0, 100, 100);
        let result = table.clone().query().stretch_x(25, 2).run().unwrap();

        for x in 100..=102 {
            assert!(present(&result, 25, 0, x, 100), "x={x} should be filled");
        }
        assert!(!present(&result, 25, 0, 103, 100));
    }

    #[test]
    fn stretch_x_wraps_across_seam() {
        // z=2 の最東セル x=3 を +1 引き延ばす → x=3 と巡回先 x=0 が埋まる。
        let table = table_with(2, 0, 3, 0);
        let result = table.clone().query().stretch_x(2, 1).run().unwrap();

        assert!(present(&result, 2, 0, 3, 0));
        assert!(present(&result, 2, 0, 0, 0));
        assert!(!present(&result, 2, 0, 1, 0));
    }

    #[test]
    fn stretch_x_full_circle_covers_all() {
        // z=2 は一周4セル。+4 引き延ばすと全周（0..=3）を覆う。
        let table = table_with(2, 0, 1, 0);
        let result = table.clone().query().stretch_x(2, 4).run().unwrap();

        for x in 0..=3 {
            assert!(present(&result, 2, 0, x, 0), "x={x} should be filled");
        }
    }

    // ---- Y方向（境界） ----

    #[test]
    fn stretch_y_negative_fills_down() {
        let table = table_with(25, 0, 100, 100);
        let result = table.clone().query().stretch_y(25, -3).run().unwrap();

        for y in 97..=100 {
            assert!(present(&result, 25, 0, 100, y), "y={y} should be filled");
        }
        assert!(!present(&result, 25, 0, 100, 96));
    }

    #[test]
    fn stretch_y_out_of_range_is_error() {
        let table = table_with(25, 0, 100, 0);
        assert!(table.clone().query().stretch_y(25, -1).run().is_err());
    }

    // ---- 総称化の確認 ----

    #[test]
    fn stretch_works_on_set() {
        let mut set = SpatialIdSet::new();
        set.insert(SingleId::new(25, 0, 100, 100).unwrap());

        let result = set.clone().query().stretch_f(25, 2).run().unwrap();

        for f in 0..=2 {
            let cell = SingleId::new(25, f, 100, 100).unwrap();
            assert!(result.get(&cell).next().is_some(), "f={f} should be filled");
        }
    }

    // 値が重なったときの衝突解決（値付き Table）。
    #[test]
    fn stretch_resolves_overlap_by_policy() {
        // f=0 に値1、f=2 に値9。両方 +2 引き延ばすと f=2 で重なる（1 由来 vs 9 由来）。
        let mut table: SpatialIdTable<u8> = SpatialIdTable::new();
        table.insert(SingleId::new(25, 0, 100, 100).unwrap(), 1u8);
        table.insert(SingleId::new(25, 2, 100, 100).unwrap(), 9u8);

        let value_at = |t: &SpatialIdTable<u8>, f: i32| {
            let cell = SingleId::new(25, f, 100, 100).unwrap();
            t.get(&cell).next().map(|(_, v)| *v)
        };

        // Max: 重なった f=2 は max(1 の伸長, 9) = 9。
        let by_max = table
            .clone()
            .query()
            .stretch_f_with(25, 2, ConflictPolicy::Max)
            .run()
            .unwrap();
        assert_eq!(value_at(&by_max, 2), Some(9));

        // Min: 重なった f=2 は min(...) = 1。
        let by_min = table
            .clone()
            .query()
            .stretch_f_with(25, 2, ConflictPolicy::Min)
            .run()
            .unwrap();
        assert_eq!(value_at(&by_min, 2), Some(1));
    }

    // FlexId を直接使う混在ズームの確認（X だけ粗いセルの引き延ばし）。
    #[test]
    fn stretch_x_on_coarse_cell() {
        let mut table = SpatialIdTable::new();
        table.insert(FlexId::new(2, 0, 1, 1, 2, 0).unwrap(), true); // x は z1 / index1（= z2 の 2,3）
        let result = table.clone().query().stretch_x(2, 1).run().unwrap();

        // 元の 2,3 に加え、+1 で 4→0（巡回）まで埋まる。
        for x in [0u32, 2, 3] {
            assert!(present(&result, 2, 0, x, 0), "x={x} should be filled");
        }
        assert!(!present(&result, 2, 0, 1, 0));
    }
}
