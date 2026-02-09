use crate::{
    Coordinate, Error, SingleId,
    geometry::helpers::vec2::Vec2,
    triangle::{self, Triangle},
};

use crate::geometry::helpers::vec3::Vec3;

#[derive(Debug, Clone)]
pub struct Polygon {
    points: Vec<Coordinate>,
    epsilon: f64,
}

impl Polygon {
    pub const DEFAULT_EPSILON: f64 = 1.0e-7;

    pub fn new(coords: Vec<Coordinate>) -> Result<Self, Error> {
        Self::new_with_epsilon(coords, Self::DEFAULT_EPSILON)
    }

    pub fn new_with_epsilon(coords: Vec<Coordinate>, epsilon: f64) -> Result<Self, Error> {
        //点が4点以上あることを確認する
        if coords.len() < 4 {
            return Err(Error::TooFewPoints(coords.len()));
        }

        //誤差範囲にある点をマージする
        let mut cleaned = Vec::with_capacity(coords.len());
        if let Some(first) = coords.first() {
            cleaned.push(first.clone());
            for i in 1..coords.len() {
                let prev = &cleaned[cleaned.len() - 1];
                let curr = &coords[i];

                let dist_sq = (prev.as_longitude() - curr.as_longitude()).powi(2)
                    + (prev.as_latitude() - curr.as_latitude()).powi(2)
                    + (prev.as_altitude() - curr.as_altitude()).powi(2);

                if dist_sq > epsilon * epsilon {
                    cleaned.push(curr.clone());
                }
            }
        }

        //視点と終点のチェック
        if cleaned.len() > 1 {
            let first = &cleaned[0];
            let last = &cleaned[cleaned.len() - 1];
            let dist_sq = (first.as_longitude() - last.as_longitude()).powi(2)
                + (first.as_latitude() - last.as_latitude()).powi(2)
                + (first.as_altitude() - last.as_altitude()).powi(2);

            if dist_sq > epsilon * epsilon {
                cleaned.push(first.clone());
            } else {
                let len = cleaned.len();
                cleaned[len - 1] = first.clone();
            }
        }

        //点が4点以上あることを確認する
        if cleaned.len() < 4 {
            return Err(Error::TooFewPoints(cleaned.len()));
        }

        //共線頂点の除去
        let cleaned = Self::remove_collinear_vertices(&cleaned, epsilon);
        if cleaned.len() < 4 {
            return Err(Error::TooFewPoints(cleaned.len()));
        }

        //自己交差を検出する
        if Self::has_self_intersection(&cleaned, epsilon) {
            return Err(Error::SelfIntersection);
        }

        Ok(Self {
            points: cleaned,
            epsilon,
        })
    }

    /// 共線頂点を除去する。
    /// 始点=終点のリング構造を前提とする。
    fn remove_collinear_vertices(points: &[Coordinate], epsilon: f64) -> Vec<Coordinate> {
        let n = points.len() - 1; // 末尾は始点の重複
        if n < 3 {
            return points.to_vec();
        }

        let mut keep = vec![true; n];

        for i in 0..n {
            let prev = &points[(i + n - 1) % n];
            let curr = &points[i];
            let next = &points[(i + 1) % n];

            let v1 = Vec3::new(
                curr.as_longitude() - prev.as_longitude(),
                curr.as_latitude() - prev.as_latitude(),
                curr.as_altitude() - prev.as_altitude(),
            );
            let v2 = Vec3::new(
                next.as_longitude() - curr.as_longitude(),
                next.as_latitude() - curr.as_latitude(),
                next.as_altitude() - curr.as_altitude(),
            );

            let cross = v1.cross(v2);
            let cross_len = cross.length();
            let edge_len = v1.length() * v2.length();

            if edge_len > 0.0 && cross_len / edge_len < epsilon * 10.0 {
                keep[i] = false;
            }
        }

        let mut result = Vec::new();
        for i in 0..n {
            if keep[i] {
                result.push(points[i].clone());
            }
        }

        if let Some(first) = result.first() {
            result.push(first.clone());
        }

        result
    }

    /// 自己交差を検出する。
    fn has_self_intersection(points: &[Coordinate], epsilon: f64) -> bool {
        let n = points.len() - 1;

        let normal = Self::compute_newell_normal(points);
        let (u_axis, v_axis) = Self::compute_projection_axes(normal);
        let projected: Vec<Vec2> = points
            .iter()
            .map(|p| Self::project_point(p, u_axis, v_axis))
            .collect();

        for i in 0..n {
            for j in (i + 2)..n {
                if i == 0 && j == n - 1 {
                    continue;
                }

                let a1 = projected[i];
                let a2 = projected[i + 1];
                let b1 = projected[j];
                let b2 = projected[j + 1];

                if Self::segments_intersect(a1, a2, b1, b2, epsilon) {
                    return true;
                }
            }
        }

        false
    }

    fn segments_intersect(a1: Vec2, a2: Vec2, b1: Vec2, b2: Vec2, epsilon: f64) -> bool {
        let d1 = Self::cross_2d(a1, a2, b1);
        let d2 = Self::cross_2d(a1, a2, b2);
        let d3 = Self::cross_2d(b1, b2, a1);
        let d4 = Self::cross_2d(b1, b2, a2);

        if ((d1 > epsilon && d2 < -epsilon) || (d1 < -epsilon && d2 > epsilon))
            && ((d3 > epsilon && d4 < -epsilon) || (d3 < -epsilon && d4 > epsilon))
        {
            return true;
        }

        false
    }

    fn cross_2d(a: Vec2, b: Vec2, c: Vec2) -> f64 {
        (b.x() - a.x()) * (c.y() - a.y()) - (b.y() - a.y()) * (c.x() - a.x())
    }

    fn compute_newell_normal(points: &[Coordinate]) -> Vec3 {
        let mut nx = 0.0;
        let mut ny = 0.0;
        let mut nz = 0.0;

        for i in 0..points.len() - 1 {
            let curr = &points[i];
            let next = &points[i + 1];

            let x0 = curr.as_longitude();
            let y0 = curr.as_latitude();
            let z0 = curr.as_altitude();

            let x1 = next.as_longitude();
            let y1 = next.as_latitude();
            let z1 = next.as_altitude();

            nx += (y0 - y1) * (z0 + z1);
            ny += (z0 - z1) * (x0 + x1);
            nz += (x0 - x1) * (y0 + y1);
        }

        Vec3::new(nx, ny, nz)
    }

    fn compute_projection_axes(normal: Vec3) -> (Vec3, Vec3) {
        let n = match normal.normalize() {
            Some(n) => n,
            None => {
                return (Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
            }
        };

        let reference = if n.x().abs() < 0.9 {
            Vec3::new(1.0, 0.0, 0.0)
        } else {
            Vec3::new(0.0, 1.0, 0.0)
        };

        let u = match n.cross(reference).normalize() {
            Some(u) => u,
            None => {
                return (Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
            }
        };

        let v = n.cross(u);

        (u, v)
    }

    fn project_point(coord: &Coordinate, u_axis: Vec3, v_axis: Vec3) -> Vec2 {
        let p = Vec3::new(
            coord.as_longitude(),
            coord.as_latitude(),
            coord.as_altitude(),
        );

        Vec2::new(p.dot(u_axis), p.dot(v_axis))
    }

    fn project_polygon_to_2d(points: &[Coordinate], u_axis: Vec3, v_axis: Vec3) -> Vec<Vec2> {
        points
            .iter()
            .map(|p| Self::project_point(p, u_axis, v_axis))
            .collect()
    }

    ///Polygon型を分割して[Triangle]にする関数
    pub fn triangulate(&self) -> Result<Vec<Triangle>, Error> {
        let points_3d = &self.points;
        let count = points_3d.len();
        if count < 4 {
            return Err(Error::TooFewPoints(count));
        }

        let mut indices: Vec<usize> = (0..count - 1).collect();

        let normal = Self::compute_newell_normal(points_3d);
        let (u_axis, v_axis) = Self::compute_projection_axes(normal);
        let points_2d = Self::project_polygon_to_2d(points_3d, u_axis, v_axis);

        let area = Self::calculate_signed_area(&points_2d, &indices);
        if area < 0.0 {
            indices.reverse();
        }

        let mut is_convex_cache: Vec<bool> = (0..indices.len())
            .map(|i| {
                let n = indices.len();
                let prev = indices[(i + n - 1) % n];
                let curr = indices[i];
                let next = indices[(i + 1) % n];
                Self::is_convex(&points_2d, prev, curr, next, self.epsilon)
            })
            .collect();

        let mut triangles = Vec::with_capacity(indices.len() - 2);
        let mut loop_count = 0;
        let max_loops = indices.len() * indices.len() * 2;

        while indices.len() > 3 {
            if loop_count > max_loops {
                return Err(Error::TriangulationFailed);
            }
            loop_count += 1;

            let n = indices.len();
            let mut best_ear: Option<(usize, f64)> = None;

            for i in 0..n {
                if !is_convex_cache[i] {
                    continue;
                }

                let prev = indices[(i + n - 1) % n];
                let curr = indices[i];
                let next = indices[(i + 1) % n];

                if Self::is_ear_empty(&points_2d, &indices, prev, curr, next, self.epsilon) {
                    let tri_area =
                        Self::triangle_area_2d(points_2d[prev], points_2d[curr], points_2d[next]);
                    match &best_ear {
                        Some((_, best_area)) if *best_area >= tri_area => {}
                        _ => {
                            best_ear = Some((i, tri_area));
                        }
                    }
                }
            }

            if best_ear.is_none() {
                for i in 0..n {
                    if !is_convex_cache[i] {
                        continue;
                    }

                    let prev = indices[(i + n - 1) % n];
                    let curr = indices[i];
                    let next = indices[(i + 1) % n];

                    let tri_area =
                        Self::triangle_area_2d(points_2d[prev], points_2d[curr], points_2d[next]);

                    match &best_ear {
                        Some((_, best_area)) if *best_area >= tri_area => {}
                        _ => {
                            best_ear = Some((i, tri_area));
                        }
                    }
                }
            }

            if best_ear.is_none() {
                let mut max_area = -1.0;
                let mut max_idx = 0;
                for i in 0..n {
                    let prev = indices[(i + n - 1) % n];
                    let curr = indices[i];
                    let next = indices[(i + 1) % n];

                    let tri_area =
                        Self::triangle_area_2d(points_2d[prev], points_2d[curr], points_2d[next]);

                    if tri_area > max_area {
                        max_area = tri_area;
                        max_idx = i;
                    }
                }
                best_ear = Some((max_idx, max_area));
            }

            if let Some((i, _)) = best_ear {
                let prev = indices[(i + indices.len() - 1) % indices.len()];
                let curr = indices[i];
                let next = indices[(i + 1) % indices.len()];

                // 退化三角形フィルタ
                let tri_area =
                    Self::triangle_area_2d(points_2d[prev], points_2d[curr], points_2d[next]);

                if tri_area > self.epsilon * self.epsilon {
                    triangles.push(Triangle::new([
                        points_3d[prev].clone(),
                        points_3d[curr].clone(),
                        points_3d[next].clone(),
                    ])?);
                }

                indices.remove(i);
                is_convex_cache.remove(i);

                if indices.len() >= 3 {
                    let new_n = indices.len();

                    let prev_pos = if i == 0 { new_n - 1 } else { i - 1 };
                    let pp = indices[(prev_pos + new_n - 1) % new_n];
                    let pc = indices[prev_pos];
                    let pn = indices[(prev_pos + 1) % new_n];
                    is_convex_cache[prev_pos] =
                        Self::is_convex(&points_2d, pp, pc, pn, self.epsilon);

                    let next_pos = i % new_n;
                    let np = indices[(next_pos + new_n - 1) % new_n];
                    let nc = indices[next_pos];
                    let nn = indices[(next_pos + 1) % new_n];
                    is_convex_cache[next_pos] =
                        Self::is_convex(&points_2d, np, nc, nn, self.epsilon);
                }
            }
        }

        // 最後の3点
        if indices.len() == 3 {
            let tri_area = Self::triangle_area_2d(
                points_2d[indices[0]],
                points_2d[indices[1]],
                points_2d[indices[2]],
            );

            if tri_area > self.epsilon * self.epsilon {
                triangles.push(Triangle::new([
                    points_3d[indices[0]].clone(),
                    points_3d[indices[1]].clone(),
                    points_3d[indices[2]].clone(),
                ])?);
            }
        }

        if triangles.is_empty() {
            return Err(Error::TriangulationFailed);
        }

        Ok(triangles)
    }

    fn calculate_signed_area(points_2d: &[Vec2], indices: &[usize]) -> f64 {
        let mut area = 0.0;
        for i in 0..indices.len() {
            let curr = points_2d[indices[i]];
            let next = points_2d[indices[(i + 1) % indices.len()]];
            area += (next.x() - curr.x()) * (next.y() + curr.y());
        }
        -area / 2.0
    }

    fn is_convex(points: &[Vec2], prev: usize, curr: usize, next: usize, epsilon: f64) -> bool {
        let a = points[prev];
        let b = points[curr];
        let c = points[next];

        let cross = (b.x() - a.x()) * (c.y() - b.y()) - (b.y() - a.y()) * (c.x() - b.x());
        cross > -epsilon
    }

    fn is_ear_empty(
        points: &[Vec2],
        indices: &[usize],
        prev: usize,
        curr: usize,
        next: usize,
        epsilon: f64,
    ) -> bool {
        let a = points[prev];
        let b = points[curr];
        let c = points[next];

        for &idx in indices {
            if idx == prev || idx == curr || idx == next {
                continue;
            }

            let p = points[idx];
            if Self::is_point_in_triangle_2d(p, a, b, c, epsilon) {
                return false;
            }
        }
        true
    }

    fn is_point_in_triangle_2d(p: Vec2, a: Vec2, b: Vec2, c: Vec2, epsilon: f64) -> bool {
        let cross1 = (b.x() - a.x()) * (p.y() - a.y()) - (b.y() - a.y()) * (p.x() - a.x());
        let cross2 = (c.x() - b.x()) * (p.y() - b.y()) - (c.y() - b.y()) * (p.x() - b.x());
        let cross3 = (a.x() - c.x()) * (p.y() - c.y()) - (a.y() - c.y()) * (p.x() - c.x());

        let tol = -epsilon;
        cross1 >= tol && cross2 >= tol && cross3 >= tol
    }

    fn triangle_area_2d(a: Vec2, b: Vec2, c: Vec2) -> f64 {
        ((b.x() - a.x()) * (c.y() - a.y()) - (b.y() - a.y()) * (c.x() - a.x())).abs() / 2.0
    }

    pub fn points(&self) -> &[Coordinate] {
        &self.points
    }

    pub fn single_ids(&self, z: u8) -> Result<impl Iterator<Item = SingleId>, Error> {
        let triangles = self.triangulate()?;
        let mut all_ids = std::collections::HashSet::new();
        for triangle in triangles {
            for id in triangle.single_ids(z)? {
                all_ids.insert(id);
            }
        }
        Ok(all_ids.into_iter())
    }
}
