use crate::{Coordinate, Error, SingleId, triangle::Triangle};

#[derive(Debug, Clone)]
/// 3次元空間内の同一平面上に存在する、閉じた多角形（ポリゴン）を表す型。
///
/// 作成時に下記のことを完全に保証する。
/// - 始点と終点の座標が完全に一致しており、図形が閉じていること。
/// - すべての頂点が、3次元空間内の同一平面上に存在すること。
/// - 構成する頂点が一直線上に並んでいないこと。
pub struct Polygon {
    points: Vec<Coordinate>,
}

impl Polygon {
    pub fn new(coords: Vec<Coordinate>) -> Result<Self, Error> {
        //4点以上で構成されていることを確認
        if coords.len() < 4 {
            return Err(Error::TooFewPoints(coords.len()));
        }

        let first = coords.first().unwrap();
        let last = coords.last().unwrap();

        //閉じていることを確認
        if first != last {
            return Err(Error::NotClosedRing);
        }

        //平面であることを確認
        if !Self::is_planar(&coords)? {
            return Err(Error::NotPlanar);
        }

        // 自己交差を確認
        if Self::has_self_intersection(&coords) {
            return Err(Error::SelfIntersection);
        }

        Ok(Self { points: coords })
    }

    ///完全な面であることを検証する関数
    fn is_planar(coords: &[Coordinate]) -> Result<bool, Error> {
        if coords.len() < 4 {
            return Ok(true);
        }

        let p0 = &coords[0];
        let p1 = &coords[1];
        let p2 = &coords[2];

        let normal = Self::compute_normal(p0, p1, p2)?;

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

        Ok((nx / len, ny / len, nz / len))
    }

    fn point_to_plane_distance(
        point: &Coordinate,
        plane_point: &Coordinate,
        normal: &(f64, f64, f64),
    ) -> f64 {
        let (nx, ny, nz) = normal;

        let dx = point.as_longitude() - plane_point.as_longitude();
        let dy = point.as_latitude() - plane_point.as_latitude();
        let dz = point.as_altitude() - plane_point.as_altitude();

        nx * dx + ny * dy + nz * dz
    }

    ///面を構成する点を借用する関数
    pub fn points(&self) -> &[Coordinate] {
        &self.points
    }

    fn has_self_intersection(coords: &[Coordinate]) -> bool {
        let n = coords.len();
        // 閉じたポリゴンなので、最後の点（=最初の点）を除いた辺のリストを考える
        // 辺 i は (coords[i], coords[i+1])
        for i in 0..n - 1 {
            for j in i + 2..n - 1 {
                // 隣接する辺（例：辺1と辺2）は頂点を共有しているだけなのでスキップ
                // ただし、最初と最後の辺の結合部もチェック対象外にする
                if i == 0 && j == n - 2 {
                    continue;
                }

                if Self::segments_intersect(&coords[i], &coords[i + 1], &coords[j], &coords[j + 1])
                {
                    return true;
                }
            }
        }
        false
    }

    /// 2つの線分が交差するか判定（2D投影を利用）
    fn segments_intersect(a: &Coordinate, b: &Coordinate, c: &Coordinate, d: &Coordinate) -> bool {
        // 簡易的に経度(x), 緯度(y)平面で判定
        let ccw = |p1: &Coordinate, p2: &Coordinate, p3: &Coordinate| -> f64 {
            (p2.as_longitude() - p1.as_longitude()) * (p3.as_latitude() - p1.as_latitude())
                - (p2.as_latitude() - p1.as_latitude()) * (p3.as_longitude() - p1.as_longitude())
        };

        let res1 = ccw(a, b, c) * ccw(a, b, d);
        let res2 = ccw(c, d, a) * ccw(c, d, b);

        // 両方の線分がお互いを跨いでいれば交差
        res1 < 0.0 && res2 < 0.0
    }

    /// ポリゴンを三角形の集合（Triangle）に分割する。
    pub fn triangulate(&self) -> Result<Vec<Triangle>, Error> {
        let points = self.points();
        let mut vertices: Vec<Coordinate> = points[..points.len() - 1].to_vec();

        if vertices.len() < 3 {
            return Err(Error::TooFewPoints(vertices.len()));
        }

        let mut area_sum = 0.0;
        for i in 0..vertices.len() {
            let curr = &vertices[i];
            let next = &vertices[(i + 1) % vertices.len()];
            area_sum += (next.as_longitude() - curr.as_longitude())
                * (next.as_latitude() + curr.as_latitude());
        }

        let mut triangles = Vec::new();

        while vertices.len() > 3 {
            let mut ear_found = false;
            for i in 0..vertices.len() {
                let prev = (i + vertices.len() - 1) % vertices.len();
                let curr = i;
                let next = (i + 1) % vertices.len();

                // 自己交差がないことが保証されているため、
                // ここでの判定は「純粋な凸角判定」と「他の点の内包判定」だけで完結する
                if self.is_ear(&vertices[prev], &vertices[curr], &vertices[next], &vertices) {
                    triangles.push(Triangle::new([
                        vertices[prev],
                        vertices[curr],
                        vertices[next],
                    ]));
                    vertices.remove(curr);
                    ear_found = true;
                    break;
                }
            }

            if !ear_found {
                return Err(Error::TriangulationFailed);
            }
        }
        triangles.push(Triangle::new([vertices[0], vertices[1], vertices[2]]));
        Ok(triangles)
    }

    fn is_ear(
        &self,
        a: &Coordinate,
        b: &Coordinate,
        c: &Coordinate,
        all_points: &[Coordinate],
    ) -> bool {
        for p in all_points {
            if p == a || p == b || p == c {
                continue;
            }
            if self.is_point_in_triangle(p, a, b, c) {
                return false;
            }
        }
        true
    }

    fn is_point_in_triangle(
        &self,
        p: &Coordinate,
        a: &Coordinate,
        b: &Coordinate,
        c: &Coordinate,
    ) -> bool {
        let (px, py) = (p.as_longitude(), p.as_latitude());
        let (ax, ay) = (a.as_longitude(), a.as_latitude());
        let (bx, by) = (b.as_longitude(), b.as_latitude());
        let (cx, cy) = (c.as_longitude(), c.as_latitude());

        let v0 = (cx - ax, cy - ay);
        let v1 = (bx - ax, by - ay);
        let v2 = (px - ax, py - ay);

        let dot00 = v0.0 * v0.0 + v0.1 * v0.1;
        let dot01 = v0.0 * v1.0 + v0.1 * v1.1;
        let dot02 = v0.0 * v2.0 + v0.1 * v2.1;
        let dot11 = v1.0 * v1.0 + v1.1 * v1.1;
        let dot12 = v1.0 * v2.0 + v1.1 * v2.1;

        let inv_den = 1.0 / (dot00 * dot11 - dot01 * dot01);
        let u = (dot11 * dot02 - dot01 * dot12) * inv_den;
        let v = (dot00 * dot12 - dot01 * dot02) * inv_den;

        (u >= 0.0) && (v >= 0.0) && (u + v < 1.0)
    }

    ///面をSingleIdの集合に変換する関数
    pub fn single_ids(&self, z: u8)
    //-> Result<impl Iterator<Item = SingleId>, Error>
    {
        todo!()
    }

    ///面をRangeIdの集合に変換する関数
    pub fn range_ids(&self, z: u8)
    //-> Result<impl Iterator<Item = RangeId>, Error>
    {
        todo!()
    }
}
