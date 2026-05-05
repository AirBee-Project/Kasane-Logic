use crate::{
    Coordinate, Cylinder, Ecef, IterCoordinates, IterLines, IterPolygons, IterSolids,
    IterTriangles, Line, Polygon, Solid, Triangle, Vec3,
};
use std::f64::consts::PI;

impl IterSolids for Cylinder {
    fn iter_solids(&self) -> impl Iterator<Item = Solid> {
        let polygons = self.iter_polygons().collect();
        let solid = Solid::new(polygons, 1e-10).unwrap();
        std::iter::once(solid)
    }
}

impl IterPolygons for Cylinder {
    fn iter_polygons(&self) -> impl Iterator<Item = Polygon> {
        let vecs: [Vec3; 2] = [self.start.into(), self.end.into()];
        let vec_n = vecs[1] - vecs[0];
        let basis = vec_n
            .create_orthonormal_basis()
            .map(|v| v.scale(self.radius_m));
        let divide_num = 100_u32;
        let vertices: Vec<_> = (0..divide_num)
            .map(|i| {
                let theta = 2.0 * PI * i as f64 / divide_num as f64;
                let v = basis[0].scale(theta.cos()) + basis[1].scale(theta.sin());
                [vecs[0] + v, vecs[1] + v]
            })
            .collect();

        let to_coord = |v: Vec3| Coordinate::try_from(Ecef::from(v)).unwrap();

        // 側面の頂点リスト（イテレータ）を作成
        let side_surfaces = (0..divide_num).map(|i| {
            let next_i = (i + 1) % divide_num;
            let v_curr = vertices[i as usize];
            let v_next = vertices[next_i as usize];

            Polygon::new(
                vec![
                    to_coord(v_curr[0]),
                    to_coord(v_next[0]),
                    to_coord(v_next[1]),
                    to_coord(v_curr[1]),
                ],
                1e-10,
            )
        });

        // 側面を全て集める
        let mut raw_surfaces: Vec<Polygon> = side_surfaces.collect();

        // 底面
        raw_surfaces.push(Polygon::new(
            vertices.iter().rev().map(|v| to_coord(v[0])).collect(),
            1e-10,
        ));
        // 上面
        raw_surfaces.push(Polygon::new(
            vertices.iter().map(|v| to_coord(v[1])).collect(),
            1e-10,
        ));

        raw_surfaces.into_iter()
    }
}

impl IterTriangles for Cylinder {
    fn iter_triangles(&self) -> impl Iterator<Item = Triangle> {
        self.iter_solids().flat_map(|s| s.iter_triangles().collect::<Vec<_>>().into_iter())
    }
}

impl IterLines for Cylinder {
    fn iter_lines(&self) -> impl Iterator<Item = Line> {
        self.iter_solids().flat_map(|s| s.iter_lines().collect::<Vec<_>>().into_iter())
    }
}

impl IterCoordinates for Cylinder {
    fn iter_coordinates(&self) -> impl Iterator<Item = Coordinate> {
        self.iter_solids().flat_map(|s| s.iter_coordinates().collect::<Vec<_>>().into_iter())
    }
}
