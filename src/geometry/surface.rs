use crate::{Coordinate, Error};

#[derive(Debug, Clone)]
pub struct Surface {
    polygon: Vec<Coordinate>,
}

impl Surface {
    pub fn new(coords: Vec<Coordinate>) -> Result<Self, Error> {
        if coords.len() < 4 {
            return Err(Error::TooFewPoints(coords.len()));
        }

        let first = coords.first().unwrap();
        let last = coords.last().unwrap();

        if first != last {
            return Err(Error::NotClosedRing);
        }

        if !Self::is_planar(&coords)? {
            return Err(Error::NotPlanar);
        }

        Ok(Self { polygon: coords })
    }

    /// 厳密な平面性チェック（体積 = 0）
    fn is_planar(coords: &[Coordinate]) -> Result<bool, Error> {
        if coords.len() < 4 {
            return Ok(true); // 3点以下は常に平面
        }

        // 最初の3点で平面を定義
        let p0 = &coords[0];
        let p1 = &coords[1];
        let p2 = &coords[2];

        // 平面の法線ベクトルを計算
        let normal = Self::compute_normal(p0, p1, p2)?;

        // 残りの全点が同じ平面上にあるかチェック
        for i in 3..coords.len() {
            let pi = &coords[i];

            // 点から平面までの距離
            let distance = Self::point_to_plane_distance(pi, p0, &normal);

            if distance.abs() > 0.0 {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// 法線ベクトルを計算
    fn compute_normal(
        p0: &Coordinate,
        p1: &Coordinate,
        p2: &Coordinate,
    ) -> Result<(f64, f64, f64), Error> {
        // ベクトル p0->p1 と p0->p2
        let v1 = (
            p1.as_longitude() - p0.as_longitude(),
            p1.as_latitude() - p0.as_latitude(),
            p1.as_altitude() - p0.as_altitude(),
        );
        let v2 = (
            p2.as_longitude() - p0.as_longitude(),
            p2.as_latitude() - p0.as_latitude(),
            p2.as_altitude() - p0.as_altitude(),
        );

        // 外積で法線ベクトルを計算
        let nx = v1.1 * v2.2 - v1.2 * v2.1;
        let ny = v1.2 * v2.0 - v1.0 * v2.2;
        let nz = v1.0 * v2.1 - v1.1 * v2.0;

        // 法線ベクトルの長さ（0なら3点が共線）
        let len = (nx * nx + ny * ny + nz * nz).sqrt();

        if len < f64::EPSILON {
            return Err(Error::CollinearPoints);
        }

        // 正規化
        Ok((nx / len, ny / len, nz / len))
    }

    /// 点から平面までの距離を計算
    /// 平面方程式: n·(p - p0) = 0
    fn point_to_plane_distance(
        point: &Coordinate,
        plane_point: &Coordinate,
        normal: &(f64, f64, f64),
    ) -> f64 {
        let (nx, ny, nz) = normal;

        // ベクトル plane_point -> point
        let dx = point.as_longitude() - plane_point.as_longitude();
        let dy = point.as_latitude() - plane_point.as_latitude();
        let dz = point.as_altitude() - plane_point.as_altitude();

        // 法線方向への射影（符号付き距離）
        nx * dx + ny * dy + nz * dz
    }

    pub fn points(&self) -> &[Coordinate] {
        &self.polygon
    }
}
