use crate::spatial_id::collection::query::Query;
use crate::{
    CellValue, ConflictPolicy, Error, FlexId, SpatialId, SpatialIdCollection, SpatialIdError,
    UnaryOperator, ZoomLevel,
};
use alloc::vec;
use alloc::vec::Vec;

/// 伝播に参加させる軸の集合。有効な軸の数で 1D（直線）/ 2D（円）/ 3D（球）が決まる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpreadAxes {
    /// 東西（X）方向へ伝播するか。
    pub x: bool,
    /// 南北（Y）方向へ伝播するか。
    pub y: bool,
    /// 高さ（F）方向へ伝播するか。
    pub f: bool,
}

impl SpreadAxes {
    /// X 軸のみ（1D）。
    pub const X: Self = Self {
        x: true,
        y: false,
        f: false,
    };
    /// Y 軸のみ（1D）。
    pub const Y: Self = Self {
        x: false,
        y: true,
        f: false,
    };
    /// F 軸のみ（1D）。
    pub const F: Self = Self {
        x: false,
        y: false,
        f: true,
    };
    /// X / Y 平面（2D・同心円）。
    pub const XY: Self = Self {
        x: true,
        y: true,
        f: false,
    };
    /// X / Y / F 全軸（3D・同心球）。
    pub const XYZ: Self = Self {
        x: true,
        y: true,
        f: true,
    };
}

/// Spread 演算子のパラメータ。中心セルの値を、指定した軸に沿って同心円（球）状に伝播させる。
pub struct SpreadParam<V> {
    /// 半径を測るズームレベル。`radius` はこのズームのセル数として解釈される
    /// （`stretch` / `level` の `z` と同じ役割）。
    pub z: u8,
    /// 伝播する半径（ズーム `z` のセル数）。中心からこのセル数以内へ広がる。
    pub radius: u32,
    /// 伝播に参加させる軸。
    pub axes: SpreadAxes,
    /// 距離に応じて値を減衰させる関数。
    ///
    /// `(中心セルの値, 中心からの距離[ズーム z のセル])` を受け取り、伝播後の値を返す。
    /// 距離 0 は中心セル自身。`None` を返したセルには伝播しない（そこで打ち切る用途にも使える）。
    pub decay: fn(&V, u32) -> Option<V>,
    /// 伝播が重なったときの解決方針。
    pub conflict: ConflictPolicy<V>,
}

impl<V> SpreadParam<V> {
    /// ズーム・半径・対象軸・減衰関数・衝突解決方針からパラメータを作る。
    pub fn new(
        z: u8,
        radius: u32,
        axes: SpreadAxes,
        decay: fn(&V, u32) -> Option<V>,
        conflict: ConflictPolicy<V>,
    ) -> Self {
        Self {
            z,
            radius,
            axes,
            decay,
            conflict,
        }
    }

    /// X / Y 平面（2D・同心円）のパラメータを作る。
    pub fn xy(
        z: u8,
        radius: u32,
        decay: fn(&V, u32) -> Option<V>,
        conflict: ConflictPolicy<V>,
    ) -> Self {
        Self::new(z, radius, SpreadAxes::XY, decay, conflict)
    }

    /// X 軸沿い（1D）のパラメータを作る。
    pub fn x(
        z: u8,
        radius: u32,
        decay: fn(&V, u32) -> Option<V>,
        conflict: ConflictPolicy<V>,
    ) -> Self {
        Self::new(z, radius, SpreadAxes::X, decay, conflict)
    }

    /// Y 軸沿い（1D）のパラメータを作る。
    pub fn y(
        z: u8,
        radius: u32,
        decay: fn(&V, u32) -> Option<V>,
        conflict: ConflictPolicy<V>,
    ) -> Self {
        Self::new(z, radius, SpreadAxes::Y, decay, conflict)
    }

    /// F（高さ）軸沿い（1D）のパラメータを作る。
    pub fn f(
        z: u8,
        radius: u32,
        decay: fn(&V, u32) -> Option<V>,
        conflict: ConflictPolicy<V>,
    ) -> Self {
        Self::new(z, radius, SpreadAxes::F, decay, conflict)
    }

    /// X / Y / F 全軸（3D・同心球）のパラメータを作る。
    pub fn xyz(
        z: u8,
        radius: u32,
        decay: fn(&V, u32) -> Option<V>,
        conflict: ConflictPolicy<V>,
    ) -> Self {
        Self::new(z, radius, SpreadAxes::XYZ, decay, conflict)
    }
}

/// 各セルの値を、指定した軸に沿って同心円（球）状に周囲へ伝播させる単項演算。
///
/// 中心セルからのユークリッド距離（ズーム `z` のセル数）が `radius` 以内の各セルへ、`decay`
/// で減衰させた値を書き込む。対象軸が1つなら軸沿いの直線、2つなら円、3つなら球になる。
/// `z` がセルのズームより細かい軸は、`shift_*` と同じく z のセルへ細分化されて広がる。
/// 重なりは `conflict` で解決する（[`Query`] の既定は [`ConflictPolicy::Max`]）。
/// X 方向は地球を周回するため巡回し、Y / F が範囲外になるセルは捨てる。
pub struct Spread;

impl<A: CellValue> UnaryOperator<A> for Spread {
    type CustomParameter = SpreadParam<A>;
    type ResultValue = A;

    fn execution<S, O>(a: S, param: Self::CustomParameter) -> Result<O, Error>
    where
        S: SpatialIdCollection<Value = A>,
        O: SpatialIdCollection<Value = A>,
    {
        let cells: Vec<(FlexId, A)> = a.scan().collect();
        let spread = map_spread(cells, &param)?;
        Ok(O::from_cells(spread, &param.conflict))
    }

    fn is_identity(_param: &Self::CustomParameter) -> bool {
        // 減衰関数は中心の値すら変えうるため、恒等とは見なさない。
        false
    }
}

/// 各セルを同心円（球）状に展開し、`(FlexId, 減衰後の値)` の列へ平坦化する。
fn map_spread<A: CellValue>(
    cells: Vec<(FlexId, A)>,
    param: &SpreadParam<A>,
) -> Result<Vec<(FlexId, A)>, Error> {
    if param.z > ZoomLevel::MAX.get() {
        return Err(SpatialIdError::ZOutOfRange { z: param.z }.into());
    }

    #[cfg(feature = "rayon")]
    {
        use rayon::prelude::*;
        cells
            .into_par_iter()
            .map(|(id, value)| expand(id, value, param))
            .collect::<Result<Vec<Vec<_>>, Error>>()
            .map(|grouped| grouped.into_iter().flatten().collect())
    }

    #[cfg(not(feature = "rayon"))]
    {
        let mut out = Vec::new();
        for (id, value) in cells {
            out.extend(expand(id, value, param)?);
        }
        Ok(out)
    }
}

/// 1 つのセルを中心に、半径以内の各セルへ減衰値を割り当てた列を返す。
///
/// 距離・半径は zoom `z` のセル数で測る。`z` がセルのズームより細かい軸は、`shift_*` と
/// 同じく z のセルへ細分化されて複数セルへ広がる（`stretch` / `level` と整合）。
fn expand<A: CellValue>(
    center: FlexId,
    value: A,
    param: &SpreadParam<A>,
) -> Result<Vec<(FlexId, A)>, Error> {
    let z = param.z;
    let r = param.radius as i32;
    let r_sq = param.radius * param.radius;
    let axes = param.axes;

    let (xz, yz, fz) = (
        center.x_zoomlevel(),
        center.y_zoomlevel(),
        center.f_zoomlevel(),
    );

    // 有効な軸がすべて z 以下（= 細分化が不要）なら、move による高速経路を使う。
    let fast = (!axes.x || z <= xz) && (!axes.y || z <= yz) && (!axes.f || z <= fz);

    // 無効な軸はオフセット 0 のみ（その軸方向へは広げない）。
    let rx = if axes.x { r } else { 0 };
    let ry = if axes.y { r } else { 0 };
    let rf = if axes.f { r } else { 0 };

    let mut out = Vec::new();
    for df in -rf..=rf {
        for dy in -ry..=ry {
            for dx in -rx..=rx {
                let dist_sq = (dx * dx + dy * dy + df * df) as u32;
                if dist_sq > r_sq {
                    continue; // 円（球）の外側へは伝播しない
                }
                let Some(new_value) = (param.decay)(&value, dist_sq.isqrt()) else {
                    continue;
                };

                if fast {
                    // 1 つの z セルはセル自身のズームの `1 << (軸ズーム - z)` インデックス分。
                    let mut id = center.clone();
                    if axes.x {
                        id.move_x(dx * (1_i32 << (xz - z)));
                    }
                    if axes.y && id.move_y(dy * (1_i32 << (yz - z))).is_err() {
                        continue; // Y が範囲外（極を越える）
                    }
                    if axes.f && id.move_f(df * (1_i32 << (fz - z))).is_err() {
                        continue; // F が範囲外
                    }
                    out.push((id, new_value));
                } else {
                    // z がセルより細かい軸を含む：shift で z セルへ細分化しながら移動する。
                    let mut cur = vec![center.clone()];
                    if axes.x {
                        let mut next = Vec::new();
                        for id in &cur {
                            next.extend(id.shift_x(z, dx)?);
                        }
                        cur = next;
                    }
                    if axes.y {
                        let mut next = Vec::new();
                        for id in cur {
                            if let Ok(it) = id.shift_y(z, dy) {
                                next.extend(it);
                            }
                        }
                        cur = next;
                    }
                    if axes.f {
                        let mut next = Vec::new();
                        for id in cur {
                            if let Ok(it) = id.shift_f(z, df) {
                                next.extend(it);
                            }
                        }
                        cur = next;
                    }
                    for id in cur {
                        out.push((id, new_value.clone()));
                    }
                }
            }
        }
    }
    Ok(out)
}

impl<C: SpatialIdCollection> Query<C>
where
    C::Value: 'static,
{
    /// 対象軸・衝突方針を明示して伝播する（最も一般的な入口）。
    pub fn spread_axes_with(
        self,
        z: u8,
        radius: u32,
        axes: SpreadAxes,
        decay: fn(&C::Value, u32) -> Option<C::Value>,
        conflict: ConflictPolicy<C::Value>,
    ) -> Self {
        Query::Unary(
            crate::spatial_id::collection::query::ops::unary::UnaryOp::Spread(SpreadParam::new(
                z, radius, axes, decay, conflict,
            )),
            alloc::boxed::Box::new(self),
        )
    }

    /// X / Y 平面へ同心円状に伝播する（重なりは [`ConflictPolicy::Max`]）。
    pub fn spread(self, z: u8, radius: u32, decay: fn(&C::Value, u32) -> Option<C::Value>) -> Self {
        self.spread_with(z, radius, decay, ConflictPolicy::Max)
    }

    /// 衝突解決方針を指定できる、XY 平面への [`spread`](Self::spread)。
    pub fn spread_with(
        self,
        z: u8,
        radius: u32,
        decay: fn(&C::Value, u32) -> Option<C::Value>,
        conflict: ConflictPolicy<C::Value>,
    ) -> Self {
        self.spread_axes_with(z, radius, SpreadAxes::XY, decay, conflict)
    }

    /// X 軸沿い（1D）に伝播する（重なりは [`ConflictPolicy::Max`]）。
    pub fn spread_x(
        self,
        z: u8,
        radius: u32,
        decay: fn(&C::Value, u32) -> Option<C::Value>,
    ) -> Self {
        self.spread_x_with(z, radius, decay, ConflictPolicy::Max)
    }

    /// 衝突解決方針を指定できる、X 軸沿いの [`spread_x`](Self::spread_x)。
    pub fn spread_x_with(
        self,
        z: u8,
        radius: u32,
        decay: fn(&C::Value, u32) -> Option<C::Value>,
        conflict: ConflictPolicy<C::Value>,
    ) -> Self {
        self.spread_axes_with(z, radius, SpreadAxes::X, decay, conflict)
    }

    /// Y 軸沿い（1D）に伝播する（重なりは [`ConflictPolicy::Max`]）。
    pub fn spread_y(
        self,
        z: u8,
        radius: u32,
        decay: fn(&C::Value, u32) -> Option<C::Value>,
    ) -> Self {
        self.spread_y_with(z, radius, decay, ConflictPolicy::Max)
    }

    /// 衝突解決方針を指定できる、Y 軸沿いの [`spread_y`](Self::spread_y)。
    pub fn spread_y_with(
        self,
        z: u8,
        radius: u32,
        decay: fn(&C::Value, u32) -> Option<C::Value>,
        conflict: ConflictPolicy<C::Value>,
    ) -> Self {
        self.spread_axes_with(z, radius, SpreadAxes::Y, decay, conflict)
    }

    /// F（高さ）軸沿い（1D）に伝播する（重なりは [`ConflictPolicy::Max`]）。
    pub fn spread_f(
        self,
        z: u8,
        radius: u32,
        decay: fn(&C::Value, u32) -> Option<C::Value>,
    ) -> Self {
        self.spread_f_with(z, radius, decay, ConflictPolicy::Max)
    }

    /// 衝突解決方針を指定できる、F 軸沿いの [`spread_f`](Self::spread_f)。
    pub fn spread_f_with(
        self,
        z: u8,
        radius: u32,
        decay: fn(&C::Value, u32) -> Option<C::Value>,
        conflict: ConflictPolicy<C::Value>,
    ) -> Self {
        self.spread_axes_with(z, radius, SpreadAxes::F, decay, conflict)
    }

    /// X / Y / F 全軸へ同心球状（3D）に伝播する（重なりは [`ConflictPolicy::Max`]）。
    pub fn spread_xyz(
        self,
        z: u8,
        radius: u32,
        decay: fn(&C::Value, u32) -> Option<C::Value>,
    ) -> Self {
        self.spread_xyz_with(z, radius, decay, ConflictPolicy::Max)
    }

    /// 衝突解決方針を指定できる、3D 球の [`spread_xyz`](Self::spread_xyz)。
    pub fn spread_xyz_with(
        self,
        z: u8,
        radius: u32,
        decay: fn(&C::Value, u32) -> Option<C::Value>,
        conflict: ConflictPolicy<C::Value>,
    ) -> Self {
        self.spread_axes_with(z, radius, SpreadAxes::XYZ, decay, conflict)
    }
}

#[cfg(test)]
mod tests {
    use crate::SpatialIdCollection;
    use crate::{ConflictPolicy, SingleId, SpatialIdSet, SpatialIdTable};

    fn table_with(z: u8, f: i32, x: u32, y: u32, v: u8) -> SpatialIdTable<u8> {
        let mut table = SpatialIdTable::new();
        table.insert(SingleId::new(z, f, x, y).unwrap(), v);
        table
    }

    fn value_at(table: &SpatialIdTable<u8>, z: u8, f: i32, x: u32, y: u32) -> Option<u8> {
        let cell = SingleId::new(z, f, x, y).unwrap();
        table.get(&cell).next().map(|(_, v)| *v)
    }

    /// F 軸沿い（1D）の伝播：高さ方向にだけ広がり、X / Y には広がらない。
    #[test]
    fn spread_f_propagates_only_along_height() {
        let table = table_with(25, 0, 100, 100, 100);
        let result = table
            .clone()
            .query()
            .spread_f(25, 2, |v, dist| {
                let d = v.saturating_sub((dist * 10) as u8);
                (d > 0).then_some(d)
            })
            .run()
            .unwrap();

        // F 方向には ±2 まで減衰しながら広がる。
        assert_eq!(value_at(&result, 25, 0, 100, 100), Some(100));
        assert_eq!(value_at(&result, 25, 1, 100, 100), Some(90));
        assert_eq!(value_at(&result, 25, -2, 100, 100), Some(80));
        assert_eq!(value_at(&result, 25, 3, 100, 100), None);
        // X / Y へは広がらない。
        assert_eq!(value_at(&result, 25, 0, 101, 100), None);
        assert_eq!(value_at(&result, 25, 0, 100, 101), None);
    }

    /// X 軸沿い（1D）の伝播：X にだけ広がり、Y / F には広がらない。
    #[test]
    fn spread_x_propagates_only_along_x() {
        let table = table_with(25, 0, 100, 100, 50);
        let result = table
            .clone()
            .query()
            .spread_x(25, 1, |v, _| Some(*v))
            .run()
            .unwrap();

        assert_eq!(value_at(&result, 25, 0, 101, 100), Some(50));
        assert_eq!(value_at(&result, 25, 0, 99, 100), Some(50));
        assert_eq!(value_at(&result, 25, 0, 100, 101), None);
        assert_eq!(value_at(&result, 25, 1, 100, 100), None);
    }

    /// 3D 球（xyz）：F 方向にも広がる。
    #[test]
    fn spread_xyz_propagates_in_all_axes() {
        let table = table_with(25, 0, 100, 100, 50);
        let result = table
            .clone()
            .query()
            .spread_xyz(25, 1, |v, _| Some(*v))
            .run()
            .unwrap();

        // 半径1の球：各軸の隣接が埋まる。
        assert_eq!(value_at(&result, 25, 0, 101, 100), Some(50));
        assert_eq!(value_at(&result, 25, 0, 100, 101), Some(50));
        assert_eq!(value_at(&result, 25, 1, 100, 100), Some(50));
        assert_eq!(value_at(&result, 25, -1, 100, 100), Some(50));
    }

    /// 軸別の `_with` で衝突方針を指定できる（spread_f_with の Min）。
    #[test]
    fn spread_f_with_resolves_overlap_by_policy() {
        // F=0 に値1、F=2 に値9。半径1で広げると F=1 で重なる。
        let mut table: SpatialIdTable<u8> = SpatialIdTable::new();
        table.insert(SingleId::new(25, 0, 100, 100).unwrap(), 1u8);
        table.insert(SingleId::new(25, 2, 100, 100).unwrap(), 9u8);

        let identity = |v: &u8, _d: u32| Some(*v);

        let by_min = table
            .clone()
            .query()
            .spread_f_with(25, 1, identity, ConflictPolicy::Min)
            .run()
            .unwrap();
        assert_eq!(value_at(&by_min, 25, 1, 100, 100), Some(1));

        let by_max = table
            .clone()
            .query()
            .spread_f_with(25, 1, identity, ConflictPolicy::Max)
            .run()
            .unwrap();
        assert_eq!(value_at(&by_max, 25, 1, 100, 100), Some(9));
    }

    /// 既定の `spread` は XY 平面のみで、F 方向には広がらない。
    #[test]
    fn spread_default_is_xy_plane_only() {
        let table = table_with(25, 5, 100, 100, 50);
        let result = table
            .clone()
            .query()
            .spread(25, 1, |v, _| Some(*v))
            .run()
            .unwrap();

        assert_eq!(value_at(&result, 25, 5, 101, 100), Some(50));
        assert_eq!(value_at(&result, 25, 4, 100, 100), None);
        assert_eq!(value_at(&result, 25, 6, 100, 100), None);
    }

    /// 中心からの距離に応じて減衰し、円の内側だけが埋まる。
    #[test]
    fn spread_fills_disc_with_decay() {
        // 半径2・1セルごとに-10で減衰（z=25 = セル自身のズーム）。
        let table = table_with(25, 0, 100, 100, 100);
        let result = table
            .clone()
            .query()
            .spread(25, 2, |v, dist| {
                let d = v.saturating_sub((dist * 10) as u8);
                (d > 0).then_some(d)
            })
            .run()
            .unwrap();

        // 中心は減衰なし。
        assert_eq!(value_at(&result, 25, 0, 100, 100), Some(100));
        // 距離1（隣接）は -10。
        assert_eq!(value_at(&result, 25, 0, 101, 100), Some(90));
        assert_eq!(value_at(&result, 25, 0, 100, 99), Some(90));
        // 距離2は -20。
        assert_eq!(value_at(&result, 25, 0, 102, 100), Some(80));
        // (2,2) はユークリッド距離 2.83 > 2 なので円の外 → 伝播しない。
        assert_eq!(value_at(&result, 25, 0, 102, 102), None);
        // 半径の外（距離3）も伝播しない。
        assert_eq!(value_at(&result, 25, 0, 103, 100), None);
    }

    /// F（高さ）は移動せず、同じ高さ面の上にだけ広がる。
    #[test]
    fn spread_keeps_height() {
        let table = table_with(25, 5, 100, 100, 50);
        let result = table
            .clone()
            .query()
            .spread(25, 1, |v, _| Some(*v))
            .run()
            .unwrap();

        assert_eq!(value_at(&result, 25, 5, 101, 100), Some(50));
        // 別の高さには漏れない。
        assert_eq!(value_at(&result, 25, 4, 101, 100), None);
        assert_eq!(value_at(&result, 25, 6, 101, 100), None);
    }

    /// `None` を返すと、そのセルには伝播しない（打ち切り）。
    #[test]
    fn spread_none_stops_propagation() {
        let table = table_with(25, 0, 100, 100, 5);
        // 距離1以上は None。
        let result = table
            .clone()
            .query()
            .spread(25, 3, |v, dist| (dist == 0).then_some(*v))
            .run()
            .unwrap();

        assert_eq!(value_at(&result, 25, 0, 100, 100), Some(5));
        assert_eq!(value_at(&result, 25, 0, 101, 100), None);
    }

    /// 重なりは ConflictPolicy で解決する（既定は Max）。
    #[test]
    fn spread_resolves_overlap_by_policy() {
        // 隣り合う2セル（値1と値9）を半径1で広げると、中間セルで重なる。
        let mut table: SpatialIdTable<u8> = SpatialIdTable::new();
        table.insert(SingleId::new(25, 0, 100, 100).unwrap(), 1u8);
        table.insert(SingleId::new(25, 0, 102, 100).unwrap(), 9u8);

        let identity = |v: &u8, _d: u32| Some(*v);

        // Max（既定）: 重なる x=101 は max(1, 9) = 9。
        let by_max = table.clone().query().spread(25, 1, identity).run().unwrap();
        assert_eq!(value_at(&by_max, 25, 0, 101, 100), Some(9));

        // Min: 重なる x=101 は min(1, 9) = 1。
        let by_min = table
            .clone()
            .query()
            .spread_with(25, 1, identity, ConflictPolicy::Min)
            .run()
            .unwrap();
        assert_eq!(value_at(&by_min, 25, 0, 101, 100), Some(1));
    }

    /// `z` がセルより粗いと、半径はその粗いセル単位で測られる（ステップが広がる）。
    #[test]
    fn spread_radius_uses_given_zoom() {
        // セルは z=25。z=24 で半径1 → 1ステップ = 2 (= 1 << (25-24)) インデックス分。
        let table = table_with(25, 0, 100, 100, 7);
        let result = table
            .clone()
            .query()
            .spread(24, 1, |v, _| Some(*v))
            .run()
            .unwrap();

        // x=102 / x=98（±2）は埋まるが、その間の x=101 は埋まらない。
        assert_eq!(value_at(&result, 25, 0, 102, 100), Some(7));
        assert_eq!(value_at(&result, 25, 0, 98, 100), Some(7));
        assert_eq!(value_at(&result, 25, 0, 101, 100), None);
    }

    /// `z` がセルより細かい場合はエラーにせず、z のセルへ細分化して伝播する。
    #[test]
    fn spread_finer_zoom_subdivides() {
        // セルは z=24。z=25（1段細かい）で伝播してもエラーにならず、結果が得られる。
        let table = table_with(24, 0, 100, 100, 7);
        let result = table
            .clone()
            .query()
            .spread(25, 1, |v, _| Some(*v))
            .run()
            .unwrap();
        assert!(!result.is_empty());
    }

    /// `z` が最大ズームを超える場合はエラー。
    #[test]
    fn spread_zoom_over_max_is_error() {
        let table = table_with(25, 0, 100, 100, 7);
        assert!(
            table
                .clone()
                .query()
                .spread(u8::MAX, 1, |v, _| Some(*v))
                .run()
                .is_err()
        );
    }

    /// 値を持たない集合（SpatialIdSet）でも動く。
    #[test]
    fn spread_works_on_set() {
        let mut set = SpatialIdSet::new();
        set.insert(SingleId::new(25, 0, 100, 100).unwrap());

        let result = set
            .clone()
            .query()
            .spread(25, 1, |_, _| Some(()))
            .run()
            .unwrap();

        // 隣接セルまで広がる。
        let neighbor = SingleId::new(25, 0, 101, 100).unwrap();
        assert!(result.get(&neighbor).next().is_some());
        // 円の外（距離2）は広がらない。
        let far = SingleId::new(25, 0, 102, 100).unwrap();
        assert!(result.get(&far).next().is_none());
    }
}
