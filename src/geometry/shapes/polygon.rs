use crate::{Coordinate, Ecef, triangle::Triangle};

#[derive(Debug, Clone)]
/// 3次元空間における多角形（ポリゴン）を表す型。
///
/// 頂点リスト（[Coordinate] のVec）によって定義される平面的な領域を表現する。
/// 生成時に頂点の重複排除などが行われ、幾何計算に適した状態に保たれる。
pub struct Polygon {
    vertices: Vec<Coordinate>,
}

impl Polygon {
    /// 頂点座標のリストから新しい [Polygon] を作成。
    ///
    /// # 処理内容
    /// - 連続して重複している頂点（`epsilon` 以内の距離）を1つに統合。
    /// - 始点と終点が重複している場合（閉じたリング）、終点を削除して開いた頂点リストに正規化。
    /// - 処理によって点の数が3未満だった場合は`Vec<Coordinate>`を空にする。
    ///
    /// # 引数
    /// - `raw_points` - ポリゴンを構成する頂点のリスト。
    /// - `epsilon` - 同一点とみなす許容誤差（メートル単位）。
    pub fn new(raw_points: Vec<Coordinate>, epsilon: f64) -> Self {
        if raw_points.is_empty() {
            return Self { vertices: vec![] };
        }

        let mut vertices: Vec<Coordinate> = Vec::new();

        for p in raw_points {
            if let Some(last) = vertices.last() {
                if !last.eq_epsilon(&p, epsilon) {
                    vertices.push(p);
                }
            } else {
                vertices.push(p);
            }
        }

        if vertices.len() > 2 {
            if vertices[0].eq_epsilon(vertices.last().unwrap(), epsilon) {
                vertices.pop();
            }
        }

        if vertices.len() < 3 {
            return Self { vertices: vec![] };
        }

        Self { vertices }
    }

    /// [Polygon]を構成する点を返す。
    pub fn vertices(&self) -> &Vec<Coordinate> {
        &self.vertices
    }

    /// [Polygon] 全体を三角形分割し、構成する [Triangle] のリストを返します。
    pub fn triangulate(&self) -> Vec<Triangle> {
        let n = self.vertices.len();
        if n < 3 {
            return vec![];
        }

        if n == 3 {
            return vec![Triangle::new([
                self.vertices[0],
                self.vertices[1],
                self.vertices[2],
            ])];
        }

        // 計算用に全て ECEF に変換
        let ecef_points: Vec<Ecef> = self.vertices.iter().map(|&c| c.into()).collect();

        // 投影軸の決定
        let (u_axis, v_axis) = Self::get_projection_axes(&ecef_points);

        // 2D投影
        let points_2d: Vec<(f64, f64)> = ecef_points
            .iter()
            .map(|p| p.project_2d(u_axis, v_axis))
            .collect();

        // 回転方向の検知
        let area = Self::signed_area(&points_2d);
        let area_sign = if area > 0.0 { 1.0 } else { -1.0 };

        // 耳切りループ
        let mut indices: Vec<usize> = (0..n).collect();
        let mut result = Vec::with_capacity(n - 2);
        let mut count = 0;
        let max_iters = n * n;

        while indices.len() > 3 && count < max_iters {
            let mut ear_found = false;
            let len = indices.len();

            for i in 0..len {
                let prev_idx = indices[(i + len - 1) % len];
                let curr_idx = indices[i];
                let next_idx = indices[(i + 1) % len];

                if Self::is_ear(
                    prev_idx, curr_idx, next_idx, &indices, &points_2d, area_sign,
                ) {
                    result.push(Triangle::new([
                        self.vertices[prev_idx],
                        self.vertices[curr_idx],
                        self.vertices[next_idx],
                    ]));
                    indices.remove(i);
                    ear_found = true;
                    break;
                }
            }
            if !ear_found {
                break;
            }
            count += 1;
        }

        if indices.len() == 3 {
            result.push(Triangle::new([
                self.vertices[indices[0]],
                self.vertices[indices[1]],
                self.vertices[indices[2]],
            ]));
        }

        result
    }

    /// Newell's Method による法線概算と投影軸の選択
    fn get_projection_axes(pts: &[Ecef]) -> (usize, usize) {
        let mut nx = 0.0;
        let mut ny = 0.0;
        let mut nz = 0.0;

        let len = pts.len();

        for i in 0..len {
            let curr = pts[i];
            let next = pts[(i + 1) % len];

            nx += (curr.as_y() - next.as_y()) * (curr.as_z() + next.as_z());
            ny += (curr.as_z() - next.as_z()) * (curr.as_x() + next.as_x());
            nz += (curr.as_x() - next.as_x()) * (curr.as_y() + next.as_y());
        }

        let ax = nx.abs();
        let ay = ny.abs();
        let az = nz.abs();

        if ax >= ay && ax >= az {
            (1, 2)
        } else if ay >= ax && ay >= az {
            (0, 2)
        } else {
            (0, 1)
        }
    }

    fn signed_area(pts: &[(f64, f64)]) -> f64 {
        let mut area = 0.0;
        for i in 0..pts.len() {
            let j = (i + 1) % pts.len();
            area += pts[i].0 * pts[j].1;
            area -= pts[j].0 * pts[i].1;
        }
        area / 2.0
    }

    fn is_ear(
        p: usize,
        c: usize,
        n: usize,
        indices: &[usize],
        pts: &[(f64, f64)],
        area_sign: f64,
    ) -> bool {
        let a = pts[p];
        let b = pts[c];
        let c_pt = pts[n];

        // 2D外積 (z成分)
        let cross = (b.0 - a.0) * (c_pt.1 - b.1) - (b.1 - a.1) * (c_pt.0 - b.0);

        // 凹判定
        if cross * area_sign <= -1e-10 {
            return false;
        }

        // 包含判定
        for &idx in indices {
            if idx == p || idx == c || idx == n {
                continue;
            }
            if Self::is_point_in_triangle(pts[idx], a, b, c_pt) {
                return false;
            }
        }
        true
    }

    fn is_point_in_triangle(p: (f64, f64), a: (f64, f64), b: (f64, f64), c: (f64, f64)) -> bool {
        let area2 = 0.5 * (-b.1 * c.0 + a.1 * (-b.0 + c.0) + a.0 * (b.1 - c.1) + b.0 * c.1);
        if area2.abs() < 1e-12 {
            return false;
        }
        let s =
            1.0 / (2.0 * area2) * (a.1 * c.0 - a.0 * c.1 + (c.1 - a.1) * p.0 + (a.0 - c.0) * p.1);
        let t =
            1.0 / (2.0 * area2) * (a.0 * b.1 - a.1 * b.0 + (a.1 - b.1) * p.0 + (b.0 - a.0) * p.1);
        s > 0.0 && t > 0.0 && (1.0 - s - t) > 0.0
    }
}
