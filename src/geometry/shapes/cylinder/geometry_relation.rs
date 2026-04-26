use crate::{
    Coordinate, Cylinder, Ecef, IntoCoordinates, IntoLines, IntoSolids, IntoTriangles, Line,
    Polygon, Solid, SpatialVector, Triangle,
};
use std::f64::consts::PI;

impl IntoSolids for Cylinder {
    fn into_solids(self) -> impl Iterator<Item = Solid> {
        let vecs: [SpatialVector; 2] = self.points.map(|p| p.into());
        let vec_n = vecs[1] - vecs[0];
        let basis = vec_n
            .create_orthonormal_basis()
            .map(|v| v.scale(self.radius));
        let devide_num = 100_u32;
        let vertices: Vec<_> = (0..devide_num)
            .map(|i| {
                let theta = 2.0 * PI * i as f64 / devide_num as f64;
                let v = basis[0].scale(theta.cos()) + basis[1].scale(theta.sin());
                [vecs[0] + v, vecs[1] + v]
            })
            .collect();

        let to_coord = |v: SpatialVector| Coordinate::try_from(Ecef::from(v)).unwrap();

        // 側面の頂点リスト（イテレータ）を作成
        let side_surfaces = (0..devide_num).map(|i| {
            let next_i = (i + 1) % devide_num;
            let v_curr = vertices[i as usize];
            let v_next = vertices[next_i as usize];

            vec![
                to_coord(v_curr[0]),
                to_coord(v_next[0]),
                to_coord(v_next[1]),
                to_coord(v_curr[1]),
            ]
        });

        // 側面、底面、上面を全て集める
        let mut raw_surfaces: Vec<Vec<Coordinate>> = side_surfaces.collect();

        // 底面 (法線が外側を向くように順序を調整する必要がある場合は `.rev()` などを付与します)
        raw_surfaces.push(vertices.iter().map(|v| to_coord(v[0])).collect());
        // 上面
        raw_surfaces.push(vertices.iter().map(|v| to_coord(v[1])).rev().collect());

        // Solidを作成 (許容誤差epsilonは適宜調整、ここでは 1e-6 を仮置き)
        let solid = Solid::new(raw_surfaces, 1e-6).unwrap();

        // イテレータとして返す
        std::iter::once(solid)
    }
    fn iter_solids(&self) -> impl Iterator<Item = Solid> {
        self.clone().into_solids()
    }
}
