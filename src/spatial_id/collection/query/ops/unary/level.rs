use crate::spatial_id::collection::query::Query;
use crate::{CellValue, ConflictPolicy, Error, FlexId, SpatialIdCollection, UnaryOperator};
use alloc::vec;
use alloc::vec::Vec;

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

    fn execution<S, O>(a: S, param: Self::CustomParameter) -> Result<O, Error>
    where
        S: SpatialIdCollection<Value = A>,
        O: SpatialIdCollection<Value = A>,
    {
        let cells: Vec<(FlexId, A)> = a.scan().collect();
        let leveled = super::map_cells(cells, |id| expand(id.clone(), &param))?;
        Ok(O::from_cells(leveled, &param.conflict))
    }

    fn is_identity(_param: &Self::CustomParameter) -> bool {
        false
    }
}

fn expand<V>(flex_id: FlexId, param: &LevelParam<V>) -> Result<Vec<FlexId>, Error> {
    if param.x.is_none()
        && param.y.is_none()
        && let Some(f) = &param.f
    {
        return Ok(flex_id.level_f(f.z, f.lo, f.hi)?.collect());
    }
    if param.f.is_none()
        && param.y.is_none()
        && let Some(x) = &param.x
    {
        return Ok(flex_id.level_x(x.z, x.lo, x.hi)?.collect());
    }
    if param.f.is_none()
        && param.x.is_none()
        && let Some(y) = &param.y
    {
        return Ok(flex_id.level_y(y.z, y.lo, y.hi)?.collect());
    }

    if param.x.is_none()
        && param.y.is_none()
        && let Some(f) = &param.f
    {
        return Ok(flex_id.level_f(f.z, f.lo, f.hi)?.collect());
    }

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

    let mut out = Vec::new();
    for id in ids {
        out.extend(level(&id, *z, *lo, *hi)?);
    }
    Ok(out)
}

impl<C: SpatialIdCollection> Query<C>
where
    C::Value: 'static,
{
    pub fn level_f(self, z: u8, lo: i32, hi: i32) -> Self {
        self.level_f_with(z, lo, hi, ConflictPolicy::Overwrite)
    }

    pub fn level_x(self, z: u8, lo: u32, hi: u32) -> Self {
        self.level_x_with(z, lo, hi, ConflictPolicy::Overwrite)
    }

    pub fn level_y(self, z: u8, lo: u32, hi: u32) -> Self {
        self.level_y_with(z, lo, hi, ConflictPolicy::Overwrite)
    }

    pub fn level_f_with(self, z: u8, lo: i32, hi: i32, conflict: ConflictPolicy<C::Value>) -> Self {
        Query::Unary(
            crate::spatial_id::collection::query::ops::unary::UnaryOp::Level(LevelParam::f(
                z, lo, hi, conflict,
            )),
            alloc::boxed::Box::new(self),
        )
    }

    pub fn level_x_with(self, z: u8, lo: u32, hi: u32, conflict: ConflictPolicy<C::Value>) -> Self {
        Query::Unary(
            crate::spatial_id::collection::query::ops::unary::UnaryOp::Level(LevelParam::x(
                z, lo, hi, conflict,
            )),
            alloc::boxed::Box::new(self),
        )
    }

    pub fn level_y_with(self, z: u8, lo: u32, hi: u32, conflict: ConflictPolicy<C::Value>) -> Self {
        Query::Unary(
            crate::spatial_id::collection::query::ops::unary::UnaryOp::Level(LevelParam::y(
                z, lo, hi, conflict,
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
    fn level_f_fills_absolute_range() {
        // f=0 を [0,3] に揃える → 0..=3 が埋まる。
        let table = table_with(25, 0, 100, 100);
        let result = table.clone().query().level_f(25, 0, 3).run().unwrap();

        for f in 0..=3 {
            assert!(present(&result, 25, f, 100, 100), "f={f} should be filled");
        }
        assert!(!present(&result, 25, 4, 100, 100));
        assert!(!present(&result, 25, -1, 100, 100));
    }

    #[test]
    fn level_f_truncates_overflow() {
        // f=0 と f=20 のセルを [0,5] に揃える → 0..=5 のみ。20 は削られる。
        let mut table = table_with(25, 0, 100, 100);
        table.insert(SingleId::new(25, 20, 100, 100).unwrap(), true);
        let result = table.clone().query().level_f(25, 0, 5).run().unwrap();

        for f in 0..=5 {
            assert!(present(&result, 25, f, 100, 100), "f={f} should be filled");
        }
        assert!(!present(&result, 25, 6, 100, 100));
        assert!(!present(&result, 25, 20, 100, 100));
    }

    #[test]
    fn level_f_flattens_relief_across_columns() {
        // 凹凸データ: 列Aは f=0、列Bは f=5（出っ張り）。両方を [0,2] に揃えると
        // どちらの列も同じ f-範囲になり、起伏が平坦化される。
        let mut table = table_with(25, 0, 100, 100); // 列A
        table.insert(SingleId::new(25, 5, 101, 100).unwrap(), true); // 列B（高い）
        let result = table.clone().query().level_f(25, 0, 2).run().unwrap();

        for f in 0..=2 {
            assert!(present(&result, 25, f, 100, 100), "A f={f}");
            assert!(present(&result, 25, f, 101, 100), "B f={f}");
        }
        // 元の出っ張り（B の f=5）は消える。
        assert!(!present(&result, 25, 5, 101, 100));
    }

    #[test]
    fn level_f_order_independent() {
        // lo/hi を入れ替えても同じ結果。
        let table = table_with(25, 10, 100, 100);
        let asc = table.clone().query().level_f(25, 2, 6).run().unwrap();
        let desc = table.clone().query().level_f(25, 6, 2).run().unwrap();

        for f in 2..=6 {
            assert!(present(&asc, 25, f, 100, 100));
            assert!(present(&desc, 25, f, 100, 100));
        }
        assert!(!present(&asc, 25, 10, 100, 100));
    }

    #[test]
    fn level_f_out_of_range_is_error() {
        let table = table_with(25, 0, 100, 100);
        assert!(
            table
                .clone()
                .query()
                .level_f(25, 0, ZoomLevel::new(25_u8).unwrap().f_max() + 1)
                .run()
                .is_err()
        );
        assert!(
            table
                .clone()
                .query()
                .level_f(25, ZoomLevel::new(25_u8).unwrap().f_min() - 1, 0)
                .run()
                .is_err()
        );
    }

    // ---- X方向（巡回） ----

    #[test]
    fn level_x_fills_range() {
        let table = table_with(25, 0, 50, 100);
        let result = table.clone().query().level_x(25, 100, 102).run().unwrap();

        for x in 100..=102 {
            assert!(present(&result, 25, 0, x, 100), "x={x} should be filled");
        }
        assert!(!present(&result, 25, 0, 50, 100)); // 元の x=50 は捨てられる
        assert!(!present(&result, 25, 0, 103, 100));
    }

    #[test]
    fn level_x_wraps_across_seam() {
        // z=2 は一周4セル。from=3 → to=0 は境界をまたいで x=3 と x=0 を埋める。
        let table = table_with(2, 0, 1, 0);
        let result = table.clone().query().level_x(2, 3, 0).run().unwrap();

        assert!(present(&result, 2, 0, 3, 0));
        assert!(present(&result, 2, 0, 0, 0));
        assert!(!present(&result, 2, 0, 1, 0));
        assert!(!present(&result, 2, 0, 2, 0));
    }

    #[test]
    fn level_x_out_of_range_is_error() {
        let table = table_with(2, 0, 0, 0);
        assert!(
            table
                .clone()
                .query()
                .level_x(2, 0, ZoomLevel::new(2_u8).unwrap().xy_max() + 1)
                .run()
                .is_err()
        );
    }

    // ---- Y方向（境界） ----

    #[test]
    fn level_y_fills_range() {
        let table = table_with(25, 0, 100, 50);
        let result = table.clone().query().level_y(25, 100, 103).run().unwrap();

        for y in 100..=103 {
            assert!(present(&result, 25, 0, 100, y), "y={y} should be filled");
        }
        assert!(!present(&result, 25, 0, 100, 50)); // 元の y=50 は捨てられる
        assert!(!present(&result, 25, 0, 100, 104));
    }

    #[test]
    fn level_y_out_of_range_is_error() {
        let table = table_with(25, 0, 100, 0);
        assert!(
            table
                .clone()
                .query()
                .level_y(25, 0, ZoomLevel::new(25_u8).unwrap().xy_max() + 1)
                .run()
                .is_err()
        );
    }

    // ---- 総称化の確認 ----

    #[test]
    fn level_works_on_set() {
        let mut set = SpatialIdSet::new();
        set.insert(SingleId::new(25, 9, 100, 100).unwrap());

        let result = set.clone().query().level_f(25, 0, 2).run().unwrap();

        for f in 0..=2 {
            let cell = SingleId::new(25, f, 100, 100).unwrap();
            assert!(result.get(&cell).next().is_some(), "f={f} should be filled");
        }
        assert!(
            result
                .get(&SingleId::new(25, 9, 100, 100).unwrap())
                .next()
                .is_none()
        );
    }

    // 同じ列の複数セルが同じ範囲へ重なるときの衝突解決（値付き Table）。
    #[test]
    fn level_resolves_overlap_by_policy() {
        // 同一列 (x=100,y=100) の f=0 に値1、f=2 に値9。[0,3] に揃えると全 f で重なる。
        let mut table: SpatialIdTable<u8> = SpatialIdTable::new();
        table.insert(SingleId::new(25, 0, 100, 100).unwrap(), 1u8);
        table.insert(SingleId::new(25, 2, 100, 100).unwrap(), 9u8);

        let value_at = |t: &SpatialIdTable<u8>, f: i32| {
            let cell = SingleId::new(25, f, 100, 100).unwrap();
            t.get(&cell).next().map(|(_, v)| *v)
        };

        // Max: 重なったセルは max(1, 9) = 9。
        let by_max = table
            .clone()
            .query()
            .level_f_with(25, 0, 3, ConflictPolicy::Max)
            .run()
            .unwrap();
        assert_eq!(value_at(&by_max, 1), Some(9));

        // Min: 重なったセルは min(1, 9) = 1。
        let by_min = table
            .clone()
            .query()
            .level_f_with(25, 0, 3, ConflictPolicy::Min)
            .run()
            .unwrap();
        assert_eq!(value_at(&by_min, 1), Some(1));
    }

    // FlexId を直接使う混在ズームの確認（Y だけ粗いセルを範囲へ揃える）。
    #[test]
    fn level_y_on_coarse_cell() {
        let mut table = SpatialIdTable::new();
        table.insert(FlexId::new(2, 0, 2, 2, 1, 1).unwrap(), true); // y は z1 / index1（= z2 の 2,3）
        let result = table.clone().query().level_y(2, 0, 1).run().unwrap();

        // y は [0,1] へ揃えられ、元の 2,3 は消える。
        for y in 0..=1 {
            assert!(present(&result, 2, 0, 2, y), "y={y} should be filled");
        }
        assert!(!present(&result, 2, 0, 2, 2));
        assert!(!present(&result, 2, 0, 2, 3));
    }
}
