use alloc::vec;
use alloc::vec::Vec;

use crate::{
    CellValue, ConflictPolicy, Error, FlexId, SpatialId, SpatialIdCollection, SpatialIdError,
    UnaryOperator, ZoomLevel,
};

/// 集合演算をメソッドとして呼び出す拡張トレイト
pub mod ops;

#[cfg(test)]
mod tests;

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
/// 重なりは `conflict` で解決する（[`Query`](crate::spatial_id::collection::expr::query::Query) の既定は [`ConflictPolicy::Max`]）。
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
