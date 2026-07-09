use alloc::vec::Vec;

pub mod impls;
#[cfg(test)]
mod tests;
use core::f64::consts::PI;

use crate::{Coordinate, Ecef, Error, GeometryError, Polygon, Solid, Vec3, Vec3Ecef};
#[derive(Debug, Clone, Copy, PartialEq)]
/// 3次元空間における円柱を表す型。
///
/// 中心線及び半径によって定義される立体的な領域を表現する。
pub struct Cylinder {
    pub start: Coordinate,
    pub end: Coordinate,
    pub radius_m: f64,
}

impl Cylinder {
    pub fn new(start: Coordinate, end: Coordinate, radius_m: f64) -> Result<Self, Error> {
        if radius_m > 0.0 {
            Ok(Cylinder {
                start,
                end,
                radius_m,
            })
        } else {
            Err(GeometryError::RadiusNegative { radius: radius_m }.into())
        }
    }

    /// [Cylinder]を近似した[Solid]を作成する関数
    pub fn rough_solid(&self) -> Solid {
        let polygons = self.rough_surfaces().collect();
        Solid::new(polygons, 1e-10).unwrap()
    }

    /// [Cylinder]を近似した立体の表面を[Polygon]として返す関数
    pub fn rough_surfaces(&self) -> impl Iterator<Item = Polygon> {
        let vecs: [Vec3Ecef; 2] = [self.start.into(), self.end.into()];
        let vec_n = vecs[1] - vecs[0];
        let basis = vec_n
            .create_orthonormal_basis()
            .map(|v| v.scale(self.radius_m));
        let divide_num = 100_u32;
        let vertices: Vec<_> = (0..divide_num)
            .map(|i| {
                let theta = 2.0 * PI * f64::from(i) / f64::from(divide_num);
                let v = basis[0].scale(libm::cos(theta)) + basis[1].scale(libm::sin(theta));
                [vecs[0] + v, vecs[1] + v]
            })
            .collect();

        let to_coord = |v: Vec3Ecef| Coordinate::try_from(Ecef::from(v)).unwrap();

        // 底面・上面を先に構築（vertices と to_coord を消費）
        let bottom = Polygon::new(
            vertices.iter().rev().map(|v| to_coord(v[0])).collect(),
            1e-10,
        );
        let top = Polygon::new(vertices.iter().map(|v| to_coord(v[1])).collect(), 1e-10);

        // 側面の頂点リスト（イテレータ）を作成
        let side_surfaces = (0..divide_num).map(move |i| {
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

        side_surfaces.chain([bottom, top])
    }
}
