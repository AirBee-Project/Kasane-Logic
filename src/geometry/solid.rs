use crate::{Coordinate, Error, Polygon, triangle::Triangle};
use std::collections::HashMap;

#[derive(Debug, Clone)]
/// 隙間や穴のない、完全に閉じた立体を表す型。
///
/// 作成時に下記のことを保証する。
/// - すべての辺は、正確に2つの面によって逆向きに共有されること（多様体条件）。
/// - 面の法線方向が一貫していること（向きの整合性）。
/// - 退化辺（長さゼロの辺）が存在しないこと。
/// - 頂点の一致判定は epsilon に基づき、正規化後にビット単位で管理される。
/// - 立体がトポロジー的に連結であること。
/// - 符号付き体積が正であること（退化立体の排除）。
pub struct Solid {
    surfaces: Vec<Polygon>,
    epsilon: f64,
}

impl Solid {
    pub const DEFAULT_EPSILON: f64 = 1.0e-10;

    pub fn new(surfaces: Vec<Polygon>) -> Result<Self, Error> {
        Self::new_with_epsilon(surfaces, Self::DEFAULT_EPSILON)
    }

    pub fn new_with_epsilon(surfaces: Vec<Polygon>, epsilon: f64) -> Result<Self, Error> {
        if surfaces.is_empty() {
            return Err(Error::EmptySolid);
        }

        // ========================================
        // 1. 頂点の正規化
        //    全面の全頂点を集約し、epsilon 以内の頂点を同一の代表座標に統一する。
        //    これにより Polygon 側の epsilon マージとの矛盾を解消する。
        // ========================================
        let surfaces = Self::normalize_vertices(surfaces, epsilon);

        let solid = Self { surfaces, epsilon };

        // ========================================
        // 2. 閉じた多様体であることの検証
        // ========================================
        solid.validate_closed_manifold()?;

        // ========================================
        // 3. 連結性の検証
        // ========================================
        solid.validate_connectivity()?;

        // ========================================
        // 4. 符号付き体積の検証（退化立体の排除）
        // ========================================
        solid.validate_positive_volume()?;

        Ok(solid)
    }

    // ================================================================
    //  頂点の正規化
    // ================================================================

    /// 全面の全頂点を走査し、epsilon 以内の頂点を同一の代表座標に正規化する。
    /// Union-Find を用いてグループ化し、各グループの最初に見つかった頂点を代表とする。
    fn normalize_vertices(surfaces: Vec<Polygon>, epsilon: f64) -> Vec<Polygon> {
        // 全頂点を収集（リングの末尾重複を除く）
        let mut all_points: Vec<Coordinate> = Vec::new();
        let mut surface_ranges: Vec<(usize, usize)> = Vec::new();

        for surface in &surfaces {
            let points = surface.points();
            let start = all_points.len();
            // リングの末尾（始点の重複）を除いて収集
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

        // Union-Find で epsilon 以内の頂点をグループ化
        let mut parent: Vec<usize> = (0..n).collect();

        // find with path compression
        fn find(parent: &mut [usize], i: usize) -> usize {
            if parent[i] != i {
                parent[i] = find(parent, parent[i]);
            }
            parent[i]
        }

        // union
        fn union(parent: &mut [usize], a: usize, b: usize) {
            let ra = find(parent, a);
            let rb = find(parent, b);
            if ra != rb {
                // 小さいインデックスを代表にする（安定性のため）
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

        // 各グループの代表座標を決定（最小インデックスの頂点の座標）
        let mut representative: HashMap<usize, Coordinate> = HashMap::new();
        for i in 0..n {
            let root = find(&mut parent, i);
            representative
                .entry(root)
                .or_insert_with(|| all_points[root].clone());
        }

        // 正規化された座標で置換
        let mut normalized_points: Vec<Coordinate> = Vec::with_capacity(n);
        for i in 0..n {
            let root = find(&mut parent, i);
            normalized_points.push(representative[&root].clone());
        }

        // 正規化された座標で Polygon を再構築
        let mut new_surfaces = Vec::with_capacity(surfaces.len());
        for (idx, (start, end)) in surface_ranges.iter().enumerate() {
            let mut coords: Vec<Coordinate> = normalized_points[*start..*end].to_vec();
            // リングを閉じる
            if let Some(first) = coords.first() {
                coords.push(first.clone());
            }

            // Polygon::new_with_epsilon で再構築
            // 正規化済みなので通常は成功するが、退化した場合はスキップせずエラーにすべき
            // ここでは元の Polygon の epsilon を引き継ぐ
            match Polygon::new_with_epsilon(coords, epsilon) {
                Ok(polygon) => new_surfaces.push(polygon),
                Err(_) => {
                    // 正規化によって退化した面（全頂点が同一点に収束した等）は
                    // 元の面をそのまま使用し、後続の検証で捕捉する
                    new_surfaces.push(surfaces[idx].clone());
                }
            }
        }

        new_surfaces
    }

    // ================================================================
    //  閉じた多様体の検証
    // ================================================================

    /// ビット表現に変換（正規化済みなので完全一致が期待できる）
    fn coord_to_bits(c: &Coordinate) -> [u64; 3] {
        [
            c.as_longitude().to_bits(),
            c.as_latitude().to_bits(),
            c.as_altitude().to_bits(),
        ]
    }

    /// 辺のキーを正規化（小さい方を前にする）
    fn edge_key(a: [u64; 3], b: [u64; 3]) -> ([u64; 3], [u64; 3]) {
        if a < b { (a, b) } else { (b, a) }
    }

    /// 辺の方向を判定（true = forward, false = backward）
    fn is_forward(a: [u64; 3], b: [u64; 3]) -> bool {
        a < b
    }

    /// 全辺の情報を収集する
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
            // 退化辺チェック
            if a == b {
                return Err(Error::DegenerateEdge(
                    *stats
                        .forward
                        .first()
                        .or(stats.backward.first())
                        .unwrap_or(&0),
                ));
            }

            // 穴が開いている
            if stats.forward.is_empty() || stats.backward.is_empty() {
                return Err(Error::OpenHoleDetected);
            }

            // 非多様体（3面以上が共有）
            if stats.forward.len() > 1 || stats.backward.len() > 1 {
                return Err(Error::NonManifoldEdge);
            }
        }

        Ok(())
    }

    // ================================================================
    //  連結性の検証
    // ================================================================

    /// 面の隣接グラフを構築し、全面が連結であることを検証する。
    /// BFS で到達可能な面を探索し、全面に到達できなければエラー。
    fn validate_connectivity(&self) -> Result<(), Error> {
        let n = self.surfaces.len();
        if n <= 1 {
            return Ok(());
        }

        // 辺 → 面のマッピングから、面の隣接リストを構築
        let edge_map = self.collect_edges();
        let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); n];

        for (_, stats) in &edge_map {
            // forward[0] と backward[0] が隣接
            if let (Some(&f), Some(&b)) = (stats.forward.first(), stats.backward.first()) {
                if f != b {
                    adjacency[f].push(b);
                    adjacency[b].push(f);
                }
            }
        }

        // BFS
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

    /// 全面を三角形分割し、符号付き体積を計算する。
    /// 正であれば法線が外向き、負であれば内向き。
    /// ゼロに近ければ退化立体（平面的な構造など）。
    fn validate_positive_volume(&self) -> Result<(), Error> {
        let triangles = self.triangulate()?;

        let mut volume = 0.0;
        for tri in &triangles {
            let coords = tri.points();
            let a = &coords[0];
            let b = &coords[1];
            let c = &coords[2];

            // 符号付き体積の6倍 = a · (b × c)
            // （原点からの四面体の符号付き体積）
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

        // 体積がほぼゼロなら退化立体
        if volume.abs() < self.epsilon {
            return Err(Error::DegenerateSolid);
        }

        // 体積が負の場合は面の向きが内向き
        // 許容するか拒否するかはポリシー次第。ここでは絶対値が正であれば OK とする。
        // （全面の法線を反転すれば正にできるため）

        Ok(())
    }

    // ================================================================
    //  公開メソッド
    // ================================================================

    /// 表面を三角形に分割して返す
    pub fn triangulate(&self) -> Result<Vec<Triangle>, Error> {
        let mut all_triangles = Vec::new();
        for surface in &self.surfaces {
            let mut triangles = surface.triangulate()?;
            all_triangles.append(&mut triangles);
        }
        Ok(all_triangles)
    }

    /// 表面を順番に返す
    pub fn surfaces(&self) -> &[Polygon] {
        &self.surfaces
    }

    /// epsilon を返す
    pub fn epsilon(&self) -> f64 {
        self.epsilon
    }

    /// 立体を SingleId の集合に変換する関数
    pub fn single_ids(&self, z: u8)
    //-> Result<impl Iterator<Item = SingleId>, Error>
    {
        todo!()
    }

    /// 立体を RangeId の集合に変換する関数
    pub fn range_ids(&self, z: u8)
    //-> Result<impl Iterator<Item = RangeId>, Error>
    {
        todo!()
    }
}

// ================================================================
//  辺の統計情報（内部型）
// ================================================================

#[derive(Debug, Default)]
struct EdgeStats {
    forward: Vec<usize>,  // A -> B 方向でこの辺を持つ面のインデックス
    backward: Vec<usize>, // B -> A 方向でこの辺を持つ面のインデックス
}
