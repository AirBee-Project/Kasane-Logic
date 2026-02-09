use crate::{
    Coordinate, Error, Polygon,
    geometry::helpers::{aabb::AABB, vec2::Vec2, vec3::Vec3},
    triangle::Triangle,
};
use std::collections::HashMap;

/// 三角形分割結果にどの面に属するかを付与した構造体
#[derive(Debug)]
struct IndexedTriangle {
    surface_idx: usize,
    v: [Vec3; 3],
    aabb: AABB,
}

#[derive(Debug, Default)]
struct EdgeStats {
    forward: Vec<usize>,
    backward: Vec<usize>,
}

#[derive(Debug, Clone)]
/// 隙間や穴のない、完全に閉じた立体を表す型。
///
/// 作成時に下記のことを保証する。
/// - 面が1つ以上存在すること。
/// - 各面が有効な Polygon であること。
/// - 全面の頂点が epsilon で正規化されていること。
/// - すべての辺は正確に2つの面によって逆向きに共有されること（多様体条件）。
/// - 面の向きが一貫していること。
/// - 退化辺が存在しないこと。
/// - 全面がトポロジー的に連結であること。
/// - 符号付き体積がゼロでないこと。
/// - 異なる面に属する三角形同士が幾何的に貫通していないこと。
pub struct Solid {
    surfaces: Vec<Polygon>,
    epsilon: f64,
}

impl Solid {
    pub const DEFAULT_EPSILON: f64 = 1.0e-12;

    pub fn new(surfaces: Vec<Polygon>) -> Result<Self, Error> {
        Self::new_with_epsilon(surfaces, Self::DEFAULT_EPSILON)
    }

    pub fn new_with_epsilon(surfaces: Vec<Polygon>, epsilon: f64) -> Result<Self, Error> {
        if surfaces.is_empty() {
            return Err(Error::EmptySolid);
        }

        let surfaces = Self::normalize_vertices(surfaces, epsilon);

        let solid = Self { surfaces, epsilon };

        solid.validate_closed_manifold()?;
        solid.validate_connectivity()?;
        solid.validate_positive_volume()?;
        solid.validate_no_geometric_intersection()?;

        Ok(solid)
    }

    // ================================================================
    //  頂点の正規化
    // ================================================================

    fn normalize_vertices(surfaces: Vec<Polygon>, epsilon: f64) -> Vec<Polygon> {
        let mut all_points: Vec<Coordinate> = Vec::new();
        let mut surface_ranges: Vec<(usize, usize)> = Vec::new();

        for surface in &surfaces {
            let points = surface.points();
            let start = all_points.len();
            for i in 0..points.len() - 1 {
                all_points.push(points[i].clone());
            }
            let end = all_points.len();
            surface_ranges.push((start, end));
        }

        let n = all_points.len();
        if n == 0 {
            return surfaces;
        }

        let mut parent: Vec<usize> = (0..n).collect();

        fn find(parent: &mut [usize], i: usize) -> usize {
            if parent[i] != i {
                parent[i] = find(parent, parent[i]);
            }
            parent[i]
        }

        fn union(parent: &mut [usize], a: usize, b: usize) {
            let ra = find(parent, a);
            let rb = find(parent, b);
            if ra != rb {
                if ra < rb {
                    parent[rb] = ra;
                } else {
                    parent[ra] = rb;
                }
            }
        }

        let eps_sq = epsilon * epsilon;
        for i in 0..n {
            for j in (i + 1)..n {
                let pi = &all_points[i];
                let pj = &all_points[j];
                let dist_sq = (pi.as_longitude() - pj.as_longitude()).powi(2)
                    + (pi.as_latitude() - pj.as_latitude()).powi(2)
                    + (pi.as_altitude() - pj.as_altitude()).powi(2);

                if dist_sq <= eps_sq {
                    union(&mut parent, i, j);
                }
            }
        }

        let mut representative: HashMap<usize, Coordinate> = HashMap::new();
        for i in 0..n {
            let root = find(&mut parent, i);
            representative
                .entry(root)
                .or_insert_with(|| all_points[root].clone());
        }

        let mut normalized_points: Vec<Coordinate> = Vec::with_capacity(n);
        for i in 0..n {
            let root = find(&mut parent, i);
            normalized_points.push(representative[&root].clone());
        }

        let mut new_surfaces = Vec::with_capacity(surfaces.len());
        for (idx, (start, end)) in surface_ranges.iter().enumerate() {
            let mut coords: Vec<Coordinate> = normalized_points[*start..*end].to_vec();
            if let Some(first) = coords.first() {
                coords.push(first.clone());
            }

            match Polygon::new_with_epsilon(coords, epsilon) {
                Ok(polygon) => new_surfaces.push(polygon),
                Err(_) => {
                    new_surfaces.push(surfaces[idx].clone());
                }
            }
        }

        new_surfaces
    }

    // ================================================================
    //  辺の収集・検証
    // ================================================================

    fn coord_to_bits(c: &Coordinate) -> [u64; 3] {
        [
            c.as_longitude().to_bits(),
            c.as_latitude().to_bits(),
            c.as_altitude().to_bits(),
        ]
    }

    fn edge_key(a: [u64; 3], b: [u64; 3]) -> ([u64; 3], [u64; 3]) {
        if a < b { (a, b) } else { (b, a) }
    }

    fn is_forward(a: [u64; 3], b: [u64; 3]) -> bool {
        a < b
    }

    fn collect_edges(&self) -> HashMap<([u64; 3], [u64; 3]), EdgeStats> {
        let mut edge_map: HashMap<([u64; 3], [u64; 3]), EdgeStats> = HashMap::new();

        for (surface_idx, surface) in self.surfaces.iter().enumerate() {
            let points = surface.points();
            for i in 0..points.len() - 1 {
                let p1_bits = Self::coord_to_bits(&points[i]);
                let p2_bits = Self::coord_to_bits(&points[i + 1]);

                let key = Self::edge_key(p1_bits, p2_bits);
                let stats = edge_map.entry(key).or_default();

                if Self::is_forward(p1_bits, p2_bits) {
                    stats.forward.push(surface_idx);
                } else {
                    stats.backward.push(surface_idx);
                }
            }
        }

        edge_map
    }

    fn validate_closed_manifold(&self) -> Result<(), Error> {
        let edge_map = self.collect_edges();

        for ((a, b), stats) in &edge_map {
            if a == b {
                return Err(Error::DegenerateEdge(
                    *stats
                        .forward
                        .first()
                        .or(stats.backward.first())
                        .unwrap_or(&0),
                ));
            }
            if stats.forward.is_empty() || stats.backward.is_empty() {
                return Err(Error::OpenHoleDetected);
            }
            if stats.forward.len() > 1 || stats.backward.len() > 1 {
                return Err(Error::NonManifoldEdge);
            }
        }

        Ok(())
    }

    // ================================================================
    //  連結性の検証
    // ================================================================

    fn validate_connectivity(&self) -> Result<(), Error> {
        let n = self.surfaces.len();
        if n <= 1 {
            return Ok(());
        }

        let edge_map = self.collect_edges();
        let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); n];

        for (_, stats) in &edge_map {
            if let (Some(&f), Some(&b)) = (stats.forward.first(), stats.backward.first()) {
                if f != b {
                    adjacency[f].push(b);
                    adjacency[b].push(f);
                }
            }
        }

        let mut visited = vec![false; n];
        let mut queue = std::collections::VecDeque::new();
        visited[0] = true;
        queue.push_back(0);

        while let Some(current) = queue.pop_front() {
            for &neighbor in &adjacency[current] {
                if !visited[neighbor] {
                    visited[neighbor] = true;
                    queue.push_back(neighbor);
                }
            }
        }

        if visited.iter().any(|&v| !v) {
            return Err(Error::DisconnectedSolid);
        }

        Ok(())
    }

    // ================================================================
    //  符号付き体積の検証
    // ================================================================

    fn validate_positive_volume(&self) -> Result<(), Error> {
        let triangles = self.triangulate()?;

        let mut volume = 0.0;
        for tri in &triangles {
            let coords = tri.points();
            let a = &coords[0];
            let b = &coords[1];
            let c = &coords[2];

            let ax = a.as_longitude();
            let ay = a.as_latitude();
            let az = a.as_altitude();
            let bx = b.as_longitude();
            let by = b.as_latitude();
            let bz = b.as_altitude();
            let cx = c.as_longitude();
            let cy = c.as_latitude();
            let cz = c.as_altitude();

            volume +=
                ax * (by * cz - bz * cy) + ay * (bz * cx - bx * cz) + az * (bx * cy - by * cx);
        }

        volume /= 6.0;

        if volume.abs() < self.epsilon {
            return Err(Error::DegenerateSolid);
        }

        Ok(())
    }

    // ================================================================
    //  幾何的交差の検証
    // ================================================================

    /// 異なる面に属する三角形同士が幾何的に貫通していないことを検証する。
    ///
    /// 手順:
    /// 1. 全面を三角形分割し、各三角形にどの面に属するかを記録する
    /// 2. 隣接面ペア（辺を共有する面）を収集する
    /// 3. 全三角形ペアについて AABB で高速フィルタリングする
    /// 4. AABB が重なるペアについて Möller の三角形交差判定を行う
    /// 5. 隣接面の三角形同士は共有頂点での接触を除外する
    fn validate_no_geometric_intersection(&self) -> Result<(), Error> {
        // 1. 面ごとの三角形分割 + インデックス付与
        let indexed_triangles = self.build_indexed_triangles()?;

        if indexed_triangles.len() < 2 {
            return Ok(());
        }

        // 2. 隣接面ペアの収集（辺を共有する面の組み合わせ）
        let adjacent_pairs = self.collect_adjacent_surface_pairs();

        // 3. 全ペアの交差判定
        for i in 0..indexed_triangles.len() {
            for j in (i + 1)..indexed_triangles.len() {
                let tri_a = &indexed_triangles[i];
                let tri_b = &indexed_triangles[j];

                // 同一面の三角形はスキップ
                if tri_a.surface_idx == tri_b.surface_idx {
                    continue;
                }

                // AABB の事前フィルタ
                if !tri_a.aabb.intersects(&tri_b.aabb, self.epsilon) {
                    continue;
                }

                // 隣接面の三角形は共有要素での接触を許容する
                let are_adjacent = adjacent_pairs
                    .contains(&Self::surface_pair(tri_a.surface_idx, tri_b.surface_idx));

                if Self::triangles_intersect_3d(&tri_a.v, &tri_b.v, are_adjacent, self.epsilon) {
                    return Err(Error::GeometricIntersection);
                }
            }
        }

        Ok(())
    }

    /// 面ペアのキーを正規化（小さい方を前に）
    fn surface_pair(a: usize, b: usize) -> (usize, usize) {
        if a < b { (a, b) } else { (b, a) }
    }

    /// 全面を三角形分割し、面のインデックスを付与して返す
    fn build_indexed_triangles(&self) -> Result<Vec<IndexedTriangle>, Error> {
        let mut result = Vec::new();

        for (surface_idx, surface) in self.surfaces.iter().enumerate() {
            let triangles = surface.triangulate()?;
            for tri in &triangles {
                let coords = tri.points();
                let v0 = Vec3::from_coord(&coords[0]);
                let v1 = Vec3::from_coord(&coords[1]);
                let v2 = Vec3::from_coord(&coords[2]);

                result.push(IndexedTriangle {
                    surface_idx,
                    aabb: AABB::from_triangle(v0, v1, v2),
                    v: [v0, v1, v2],
                });
            }
        }

        Ok(result)
    }

    /// 辺を共有する面のペアを収集する
    fn collect_adjacent_surface_pairs(&self) -> std::collections::HashSet<(usize, usize)> {
        let edge_map = self.collect_edges();
        let mut pairs = std::collections::HashSet::new();

        for (_, stats) in &edge_map {
            if let (Some(&f), Some(&b)) = (stats.forward.first(), stats.backward.first()) {
                if f != b {
                    pairs.insert(Self::surface_pair(f, b));
                }
            }
        }

        pairs
    }

    // ================================================================
    //  三角形-三角形 交差判定（3D）
    //  Möller の分離軸ベースアルゴリズム
    // ================================================================

    /// 2つの三角形が3D空間で交差（貫通）しているかを判定する。
    ///
    /// `are_adjacent` が true の場合、頂点や辺を共有する接触は貫通とみなさない。
    fn triangles_intersect_3d(
        tri_a: &[Vec3; 3],
        tri_b: &[Vec3; 3],
        are_adjacent: bool,
        epsilon: f64,
    ) -> bool {
        // 隣接面の場合、共有頂点の数を数える
        if are_adjacent {
            let shared = Self::count_shared_vertices(tri_a, tri_b, epsilon);
            // 2頂点以上を共有 → 辺を共有しているだけ（接触であり貫通ではない）
            if shared >= 2 {
                return false;
            }
        }

        // --- Möller の三角形交差判定 ---
        // 各三角形の平面を計算し、もう一方の三角形の頂点が平面のどちら側にあるかで判定

        // 三角形Aの平面
        let e1_a = tri_a[1].sub(tri_a[0]);
        let e2_a = tri_a[2].sub(tri_a[0]);
        let n_a = e1_a.cross(e2_a);

        // 三角形Bの頂点の、平面Aに対する符号付き距離
        let db0 = n_a.dot(tri_b[0].sub(tri_a[0]));
        let db1 = n_a.dot(tri_b[1].sub(tri_a[0]));
        let db2 = n_a.dot(tri_b[2].sub(tri_a[0]));

        // 浮動小数点誤差の丸め
        let db0 = if db0.abs() < epsilon { 0.0 } else { db0 };
        let db1 = if db1.abs() < epsilon { 0.0 } else { db1 };
        let db2 = if db2.abs() < epsilon { 0.0 } else { db2 };

        // 三角形Bの全頂点が平面Aの同じ側にある → 交差しない
        if db0 > 0.0 && db1 > 0.0 && db2 > 0.0 {
            return false;
        }
        if db0 < 0.0 && db1 < 0.0 && db2 < 0.0 {
            return false;
        }

        // 三角形Bの平面
        let e1_b = tri_b[1].sub(tri_b[0]);
        let e2_b = tri_b[2].sub(tri_b[0]);
        let n_b = e1_b.cross(e2_b);

        // 三角形Aの頂点の、平面Bに対する符号付き距離
        let da0 = n_b.dot(tri_a[0].sub(tri_b[0]));
        let da1 = n_b.dot(tri_a[1].sub(tri_b[0]));
        let da2 = n_b.dot(tri_a[2].sub(tri_b[0]));

        let da0 = if da0.abs() < epsilon { 0.0 } else { da0 };
        let da1 = if da1.abs() < epsilon { 0.0 } else { da1 };
        let da2 = if da2.abs() < epsilon { 0.0 } else { da2 };

        // 三角形Aの全頂点が平面Bの同じ側にある → 交差しない
        if da0 > 0.0 && da1 > 0.0 && da2 > 0.0 {
            return false;
        }
        if da0 < 0.0 && da1 < 0.0 && da2 < 0.0 {
            return false;
        }

        // --- 共面の場合の処理 ---
        if db0 == 0.0 && db1 == 0.0 && db2 == 0.0 {
            return Self::coplanar_triangles_intersect(tri_a, tri_b, &n_a, are_adjacent, epsilon);
        }

        // --- 交差線の区間重複判定 ---
        // 2つの平面の交線上に、各三角形が作る区間を計算し、重複があれば交差
        let cross_dir = n_a.cross(n_b);

        // 交差線への投影軸を決定（cross_dir の最大成分の軸）
        let ax = cross_dir.x().abs();
        let ay = cross_dir.y().abs();
        let az = cross_dir.z().abs();

        let project = |v: Vec3| -> f64 {
            if ax >= ay && ax >= az {
                v.x()
            } else if ay >= ax && ay >= az {
                v.y()
            } else {
                v.z()
            }
        };

        // 三角形Aの交差線上の区間
        let pa0 = project(tri_a[0]);
        let pa1 = project(tri_a[1]);
        let pa2 = project(tri_a[2]);
        let interval_a = Self::compute_intersection_interval(pa0, pa1, pa2, da0, da1, da2);

        // 三角形Bの交差線上の区間
        let pb0 = project(tri_b[0]);
        let pb1 = project(tri_b[1]);
        let pb2 = project(tri_b[2]);
        let interval_b = Self::compute_intersection_interval(pb0, pb1, pb2, db0, db1, db2);

        let (interval_a, interval_b) = match (interval_a, interval_b) {
            (Some(a), Some(b)) => (a, b),
            _ => return false,
        };

        // 区間の重なり判定
        // 隣接面で頂点を1つ共有する場合、端点での接触は許容する
        if are_adjacent {
            let shared = Self::count_shared_vertices(tri_a, tri_b, epsilon);
            if shared >= 1 {
                // 端点の接触のみ → 内部で重なっている場合のみ交差とする
                let overlap_start = interval_a.0.max(interval_b.0);
                let overlap_end = interval_a.1.min(interval_b.1);
                let overlap_len = overlap_end - overlap_start;

                // 実質的な重なり長がある場合のみ交差
                return overlap_len > epsilon;
            }
        }

        // 区間が重なっていれば交差
        interval_a.0 < interval_b.1 - epsilon && interval_b.0 < interval_a.1 - epsilon
    }

    /// 三角形が交差線上に作る区間 [t_min, t_max] を計算する。
    /// d0, d1, d2 は各頂点の平面からの符号付き距離。
    /// p0, p1, p2 は各頂点の交差線への射影値。
    fn compute_intersection_interval(
        p0: f64,
        p1: f64,
        p2: f64,
        d0: f64,
        d1: f64,
        d2: f64,
    ) -> Option<(f64, f64)> {
        // 符号が異なる頂点ペア間を線形補間して交差点を求める
        let mut ts = Vec::with_capacity(2);

        // d0 と d1 が異符号
        if (d0 > 0.0 && d1 < 0.0) || (d0 < 0.0 && d1 > 0.0) {
            let t = p0 + (p1 - p0) * d0 / (d0 - d1);
            ts.push(t);
        }

        // d0 と d2 が異符号
        if (d0 > 0.0 && d2 < 0.0) || (d0 < 0.0 && d2 > 0.0) {
            let t = p0 + (p2 - p0) * d0 / (d0 - d2);
            ts.push(t);
        }

        // d1 と d2 が異符号
        if ts.len() < 2 && ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0)) {
            let t = p1 + (p2 - p1) * d1 / (d1 - d2);
            ts.push(t);
        }

        // 頂点が平面上にある場合
        if ts.len() < 2 && d0 == 0.0 {
            ts.push(p0);
        }
        if ts.len() < 2 && d1 == 0.0 {
            ts.push(p1);
        }
        if ts.len() < 2 && d2 == 0.0 {
            ts.push(p2);
        }

        if ts.len() < 2 {
            return None;
        }

        let t_min = ts[0].min(ts[1]);
        let t_max = ts[0].max(ts[1]);

        Some((t_min, t_max))
    }

    /// 共有頂点の数を数える
    fn count_shared_vertices(tri_a: &[Vec3; 3], tri_b: &[Vec3; 3], epsilon: f64) -> usize {
        let eps_sq = epsilon * epsilon;
        let mut count = 0;

        for a in tri_a {
            for b in tri_b {
                if a.sub(*b).length_sq() <= eps_sq {
                    count += 1;
                    break; // この a に対しては1つマッチすれば十分
                }
            }
        }

        count
    }

    // ================================================================
    //  共面三角形の交差判定（2D に投影して判定）
    // ================================================================

    /// 2つの三角形が同一平面上にある場合の交差判定。
    /// 法線方向を使って2Dに投影し、辺の交差で判定する。
    fn coplanar_triangles_intersect(
        tri_a: &[Vec3; 3],
        tri_b: &[Vec3; 3],
        normal: &Vec3,
        are_adjacent: bool,
        epsilon: f64,
    ) -> bool {
        // 法線の最大成分の軸を使って2Dに投影
        let ax = normal.x().abs();
        let ay = normal.y().abs();
        let az = normal.z().abs();

        let project_2d = |v: &Vec3| -> Vec2 {
            if ax >= ay && ax >= az {
                Vec2::new(v.y(), v.z())
            } else if ay >= ax && ay >= az {
                Vec2::new(v.x(), v.z())
            } else {
                Vec2::new(v.x(), v.y())
            }
        };

        let a2d: [Vec2; 3] = [
            project_2d(&tri_a[0]),
            project_2d(&tri_a[1]),
            project_2d(&tri_a[2]),
        ];
        let b2d: [Vec2; 3] = [
            project_2d(&tri_b[0]),
            project_2d(&tri_b[1]),
            project_2d(&tri_b[2]),
        ];

        // 隣接面で辺を共有している場合、共有辺以外での交差のみを検出
        if are_adjacent {
            let shared = Self::count_shared_vertices(tri_a, tri_b, epsilon);
            if shared >= 2 {
                return false;
            }
        }

        // 辺-辺の交差判定
        let edges_a = [(0, 1), (1, 2), (2, 0)];
        let edges_b = [(0, 1), (1, 2), (2, 0)];

        for &(a0, a1) in &edges_a {
            for &(b0, b1) in &edges_b {
                if are_adjacent {
                    // 共有頂点を含む辺同士の端点接触はスキップ
                    let a0_shared = Self::is_vertex_shared(&tri_a[a0], tri_b, epsilon);
                    let a1_shared = Self::is_vertex_shared(&tri_a[a1], tri_b, epsilon);
                    let b0_shared = Self::is_vertex_shared(&tri_b[b0], tri_a, epsilon);
                    let b1_shared = Self::is_vertex_shared(&tri_b[b1], tri_a, epsilon);

                    if (a0_shared || a1_shared) && (b0_shared || b1_shared) {
                        continue;
                    }
                }

                if Self::segments_intersect_2d(a2d[a0], a2d[a1], b2d[b0], b2d[b1], epsilon) {
                    return true;
                }
            }
        }

        // 一方が他方を完全に包含するケース
        if Self::point_in_triangle_2d(a2d[0], &b2d, epsilon)
            || Self::point_in_triangle_2d(b2d[0], &a2d, epsilon)
        {
            // 隣接面で頂点を共有する場合、共有頂点のみでの包含は接触
            if are_adjacent {
                let shared = Self::count_shared_vertices(tri_a, tri_b, epsilon);
                if shared >= 1 {
                    return false;
                }
            }
            return true;
        }

        false
    }

    /// 頂点が相手の三角形の頂点と一致するか
    fn is_vertex_shared(v: &Vec3, tri: &[Vec3; 3], epsilon: f64) -> bool {
        let eps_sq = epsilon * epsilon;
        tri.iter().any(|t| v.sub(*t).length_sq() <= eps_sq)
    }

    /// 2D での線分交差判定（端点での接触を除外）
    fn segments_intersect_2d(a1: Vec2, a2: Vec2, b1: Vec2, b2: Vec2, epsilon: f64) -> bool {
        let d1 = Self::cross_2d_val(a1, a2, b1);
        let d2 = Self::cross_2d_val(a1, a2, b2);
        let d3 = Self::cross_2d_val(b1, b2, a1);
        let d4 = Self::cross_2d_val(b1, b2, a2);

        if ((d1 > epsilon && d2 < -epsilon) || (d1 < -epsilon && d2 > epsilon))
            && ((d3 > epsilon && d4 < -epsilon) || (d3 < -epsilon && d4 > epsilon))
        {
            return true;
        }

        false
    }

    fn cross_2d_val(a: Vec2, b: Vec2, c: Vec2) -> f64 {
        (b.x() - a.x()) * (c.y() - a.y()) - (b.y() - a.y()) * (c.x() - a.x())
    }
    /// 点が三角形の内部にあるか（2D）
    fn point_in_triangle_2d(p: Vec2, tri: &[Vec2; 3], epsilon: f64) -> bool {
        let d1 = Self::cross_2d_val(tri[0], tri[1], p);
        let d2 = Self::cross_2d_val(tri[1], tri[2], p);
        let d3 = Self::cross_2d_val(tri[2], tri[0], p);

        let has_neg = d1 < -epsilon || d2 < -epsilon || d3 < -epsilon;
        let has_pos = d1 > epsilon || d2 > epsilon || d3 > epsilon;

        // 全て同符号なら内部（境界上は除外）
        !(has_neg && has_pos) && (d1.abs() > epsilon && d2.abs() > epsilon && d3.abs() > epsilon)
    }

    // ================================================================
    //  公開メソッド
    // ================================================================

    pub fn triangulate(&self) -> Result<Vec<Triangle>, Error> {
        let mut all_triangles = Vec::new();
        for surface in &self.surfaces {
            let mut triangles = surface.triangulate()?;
            all_triangles.append(&mut triangles);
        }
        Ok(all_triangles)
    }

    pub fn surfaces(&self) -> &[Polygon] {
        &self.surfaces
    }

    pub fn epsilon(&self) -> f64 {
        self.epsilon
    }

    pub fn single_ids(&self, z: u8)
    //-> Result<impl Iterator<Item = SingleId>, Error>
    {
        todo!()
    }

    pub fn range_ids(&self, z: u8)
    //-> Result<impl Iterator<Item = RangeId>, Error>
    {
        todo!()
    }
}
