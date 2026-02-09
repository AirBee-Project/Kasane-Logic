use crate::{Coordinate, Ecef, Error, geometry::triangle::Triangle};
#[derive(Debug, Clone)]
pub struct Surface {
    points: Vec<Coordinate>,
}

impl Surface {
    pub fn new(points: Vec<Coordinate>) -> Result<Self, Error> {
        if points.len() < 4 {
            return Err(Error::TooFewPoints(points.len()));
        }

        let first = points.first().unwrap();
        let last = points.last().unwrap();

        if first != last {
            return Err(Error::NotClosedRing);
        }
        // 1. 計算用にECEF座標（メートル単位の直交座標）へ変換
        let ecef_points: Vec<Ecef> = points.iter().map(|&p| p.into()).collect();

        let (projected_2d, _) = Self::project_to_2d(&ecef_points);

        if Self::has_self_intersection(&projected_2d) {
            return Err(Error::NonManifoldEdge);
        }

        Ok(Self { points })
    }

    pub fn points(&self) -> &[Coordinate] {
        &self.points
    }

    pub fn triangulate(&self) -> Vec<Triangle> {
        let points = &self.points;
        let count = points.len();
        if count < 4 {
            return vec![];
        }

        let ecef_points: Vec<Ecef> = points.iter().take(count - 1).map(|&p| p.into()).collect();
        let (points_2d, _) = Self::project_to_2d(&ecef_points);

        let mut indices: Vec<usize> = (0..points_2d.len()).collect();
        let mut triangles = Vec::new();

        let mut attempts = 0;
        let max_attempts = indices.len() * 2;

        while indices.len() > 3 {
            let n = indices.len();
            let mut ear_found = false;

            for i in 0..n {
                let prev_idx = indices[(i + n - 1) % n];
                let curr_idx = indices[i];
                let next_idx = indices[(i + 1) % n];

                let p_prev = points_2d[prev_idx];
                let p_curr = points_2d[curr_idx];
                let p_next = points_2d[next_idx];

                if Self::is_convex(p_prev, p_curr, p_next) {
                    let mut contains_point = false;
                    for &k in &indices {
                        if k == prev_idx || k == curr_idx || k == next_idx {
                            continue;
                        }
                        if Self::is_point_in_triangle(points_2d[k], p_prev, p_curr, p_next) {
                            contains_point = true;
                            break;
                        }
                    }

                    if !contains_point {
                        triangles.push(Triangle::new([
                            points[prev_idx],
                            points[curr_idx],
                            points[next_idx],
                        ]));

                        indices.remove(i);
                        ear_found = true;
                        break;
                    }
                }
            }

            if !ear_found {
                attempts += 1;
                if attempts > max_attempts {
                    break;
                }
            } else {
                attempts = 0;
            }
        }

        // 最後の3点
        if indices.len() == 3 {
            triangles.push(Triangle::new([
                points[indices[0]],
                points[indices[1]],
                points[indices[2]],
            ]));
        }

        triangles
    }

    /// 3次元点群を、平均法線に基づいて最適な2D平面(UV平面)に投影する
    fn project_to_2d(points: &[Ecef]) -> (Vec<(f64, f64)>, Ecef) {
        // Newell's Methodで平均法線を求める
        let mut normal = Ecef::new(0.0, 0.0, 0.0);
        for i in 0..points.len() {
            let curr = &points[i];
            let next = &points[(i + 1) % points.len()];
            normal = Ecef::new(
                normal.as_x() + (curr.as_y() - next.as_y()) * (curr.as_z() + next.as_z()),
                normal.as_y() + (curr.as_z() - next.as_z()) * (curr.as_x() + next.as_x()),
                normal.as_z() + (curr.as_x() - next.as_x()) * (curr.as_y() + next.as_y()),
            );
        }

        // 法線の正規化（長さ0なら適当な軸を設定）
        let len = (normal.as_x().powi(2) + normal.as_y().powi(2) + normal.as_z().powi(2)).sqrt();
        let normal = if len < f64::EPSILON {
            Ecef::new(0.0, 0.0, 1.0)
        } else {
            Ecef::new(
                normal.as_x() / len,
                normal.as_y() / len,
                normal.as_z() / len,
            )
        };

        // 最も法線成分が大きい軸を捨てて投影する（数値安定性のため）
        let nx = normal.as_x().abs();
        let ny = normal.as_y().abs();
        let nz = normal.as_z().abs();

        let projected = points
            .iter()
            .map(|p| {
                if nx > ny && nx > nz {
                    (p.as_y(), p.as_z()) // YZ平面
                } else if ny > nx && ny > nz {
                    (p.as_x(), p.as_z()) // XZ平面
                } else {
                    (p.as_x(), p.as_y()) // XY平面
                }
            })
            .collect();

        (projected, normal)
    }

    fn has_self_intersection(points_2d: &[(f64, f64)]) -> bool {
        let n = points_2d.len();
        for i in 0..n {
            for j in (i + 2)..n {
                if i == 0 && j == n - 1 {
                    continue;
                }

                let p1 = points_2d[i];
                let p2 = points_2d[(i + 1) % n];
                let p3 = points_2d[j];
                let p4 = points_2d[(j + 1) % n];

                if Self::segments_intersect(p1, p2, p3, p4) {
                    return true;
                }
            }
        }
        false
    }

    /// 線分交差判定 (AB と CD)
    fn segments_intersect(a: (f64, f64), b: (f64, f64), c: (f64, f64), d: (f64, f64)) -> bool {
        fn ccw(p1: (f64, f64), p2: (f64, f64), p3: (f64, f64)) -> f64 {
            (p2.0 - p1.0) * (p3.1 - p1.1) - (p2.1 - p1.1) * (p3.0 - p1.0)
        }

        let d1 = ccw(c, d, a);
        let d2 = ccw(c, d, b);
        let d3 = ccw(a, b, c);
        let d4 = ccw(a, b, d);

        // 厳密な交差（端点共有を含まない）を判定
        ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0))
            && ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0))
    }

    fn is_convex(prev: (f64, f64), curr: (f64, f64), next: (f64, f64)) -> bool {
        let cp = (curr.0 - prev.0) * (next.1 - curr.1) - (curr.1 - prev.1) * (next.0 - curr.0);
        cp >= 0.0
    }

    fn is_point_in_triangle(p: (f64, f64), a: (f64, f64), b: (f64, f64), c: (f64, f64)) -> bool {
        fn sign(p1: (f64, f64), p2: (f64, f64), p3: (f64, f64)) -> f64 {
            (p1.0 - p3.0) * (p2.1 - p3.1) - (p2.0 - p3.0) * (p1.1 - p3.1)
        }

        let d1 = sign(p, a, b);
        let d2 = sign(p, b, c);
        let d3 = sign(p, c, a);

        let has_neg = (d1 < 0.0) || (d2 < 0.0) || (d3 < 0.0);
        let has_pos = (d1 > 0.0) || (d2 > 0.0) || (d3 > 0.0);

        !(has_neg && has_pos)
    }
}
