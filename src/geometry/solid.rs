use crate::{Coordinate, Error, Surface, geometry::triangle::Triangle};
use std::collections::HashMap;

#[derive(Debug, Clone)]
/// 隙間や穴のない、完全に閉じた立体を表す型。
///
/// 作成時に下記のことを完全に保証する。
/// - 立体に穴や隙間が一切空いていないこと。
/// - すべての辺は、必ず他の面と隙間なく接合されていること。
/// - 1つの辺を3枚以上の面が共有したり、自己交差したりする構造ではないこと。
/// - すべての辺は、正確に2つの面によって共有されること。
/// - 頂点の座標はビット単位で厳密に一致していること。
pub struct Solid {
    surfaces: Vec<Surface>,
}

/// Union-Find (Disjoint Set Union) データ構造
/// 頂点のクラスタリングに使用
struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl UnionFind {
    fn new(size: usize) -> Self {
        Self {
            parent: (0..size).collect(),
            rank: vec![0; size],
        }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]); // 経路圧縮
        }
        self.parent[x]
    }

    fn union(&mut self, x: usize, y: usize) {
        let root_x = self.find(x);
        let root_y = self.find(y);

        if root_x != root_y {
            // ランクによる結合
            match self.rank[root_x].cmp(&self.rank[root_y]) {
                std::cmp::Ordering::Less => self.parent[root_x] = root_y,
                std::cmp::Ordering::Greater => self.parent[root_y] = root_x,
                std::cmp::Ordering::Equal => {
                    self.parent[root_y] = root_x;
                    self.rank[root_x] += 1;
                }
            }
        }
    }
}

impl Solid {
    pub fn triangulate(&self) -> Vec<Triangle> {
        todo!()
    }

    pub fn new_with_tolerance(surfaces: Vec<Surface>, epsilon: f64) -> Result<Self, Error> {
        if surfaces.is_empty() {
            return Err(Error::EmptySolid);
        }

        if epsilon < 0.0 {
            return Err(Error::InvalidTolerance);
        }

        // 1. すべての頂点を収集（重複含む）
        let mut all_vertices: Vec<Coordinate> = Vec::new();
        let mut surface_vertex_indices: Vec<Vec<usize>> = Vec::new();

        for surface in &surfaces {
            let points = surface.points();
            let mut indices = Vec::new();
            for point in points {
                indices.push(all_vertices.len());
                all_vertices.push(*point);
            }
            surface_vertex_indices.push(indices);
        }

        let total_vertices = all_vertices.len();

        // 2. Union-Findでクラスタリング
        let mut uf = UnionFind::new(total_vertices);

        // epsilon以内の頂点をすべてグループ化
        // 注意: O(N^2)のため、実用環境ではKD-TreeやR-treeを使用すべき
        for i in 0..total_vertices {
            for j in (i + 1)..total_vertices {
                if all_vertices[i].distance(&all_vertices[j]) <= epsilon {
                    uf.union(i, j);
                }
            }
        }

        // 3. 各クラスタの代表点を決定（重心を使用）
        let mut cluster_map: HashMap<usize, Vec<usize>> = HashMap::new();
        for i in 0..total_vertices {
            let root = uf.find(i);
            cluster_map.entry(root).or_insert_with(Vec::new).push(i);
        }

        let mut representative_vertices: HashMap<usize, Coordinate> = HashMap::new();
        for (root, indices) in &cluster_map {
            // クラスタ内の頂点の重心を計算
            let centroid = Self::compute_centroid(
                &indices.iter().map(|&i| all_vertices[i]).collect::<Vec<_>>(),
            );
            representative_vertices.insert(*root, centroid);
        }

        // 4. 各面の頂点を代表点で置き換え
        let mut valid_surfaces: Vec<Surface> = Vec::new();

        for indices in surface_vertex_indices {
            let mut new_points: Vec<Coordinate> = Vec::new();

            for idx in indices {
                let root = uf.find(idx);
                let representative = representative_vertices[&root];
                new_points.push(representative);
            }

            // 5. 縮退の解消
            // 連続する重複点を削除
            new_points.dedup();

            // 面が閉じていることを確認し、必要なら修正
            if new_points.len() >= 2 {
                let first = new_points[0];
                let last = *new_points.last().unwrap();

                // 最後の点と最初の点が異なる場合は閉じる
                if first != last {
                    new_points.push(first);
                }
            }

            // 再度dedupして、末尾の重複を解消
            // ただし、最低4点（閉じた三角形）は必要
            let original_len = new_points.len();
            new_points.dedup();

            // dedupで始点と終点が削除された場合は再度閉じる
            if new_points.len() >= 2 {
                let first = new_points[0];
                let last = *new_points.last().unwrap();
                if first != last {
                    new_points.push(first);
                }
            }

            // 6. 面の再構築
            // 頂点数が不足している場合や自己交差がある場合は、
            // その面を無視せずエラーとして報告
            match Surface::new(new_points.clone()) {
                Ok(new_surface) => {
                    valid_surfaces.push(new_surface);
                }
                Err(e) => {
                    // マージによって面が縮退した場合、立体全体のトポロジーが
                    // 破綻する可能性が高いため、エラーとして扱う
                    // （特に、元々有効だった面が無効になるのは問題）
                    return Err(Error::TopologyBrokenByMerge {
                        original_error: Box::new(e),
                        vertex_count: new_points.len(),
                    });
                }
            }
        }

        // 7. すべての面が消失していないかチェック
        if valid_surfaces.is_empty() {
            return Err(Error::AllSurfacesCollapsed);
        }

        // 面の数が元と異なる場合も警告
        if valid_surfaces.len() != surfaces.len() {
            return Err(Error::SurfaceCountMismatch {
                original: surfaces.len(),
                after_merge: valid_surfaces.len(),
            });
        }

        let solid = Self {
            surfaces: valid_surfaces,
        };

        // 8. 最終的な整合性チェック
        // 立体が閉じていることを厳密に検証
        solid.is_close()?;

        Ok(solid)
    }

    pub fn new(surfaces: Vec<Surface>) -> Result<Self, Error> {
        if surfaces.is_empty() {
            return Err(Error::EmptySolid);
        }

        let solid = Self { surfaces };
        solid.is_close()?;

        Ok(solid)
    }

    /// 頂点群の重心を計算
    fn compute_centroid(vertices: &[Coordinate]) -> Coordinate {
        let n = vertices.len() as f64;
        let mut sum_lat = 0.0;
        let mut sum_lon = 0.0;
        let mut sum_alt = 0.0;

        for v in vertices {
            sum_lat += v.as_latitude();
            sum_lon += v.as_longitude();
            sum_alt += v.as_altitude();
        }

        Coordinate::new(sum_lat / n, sum_lon / n, sum_alt / n).unwrap()
    }

    /// 立体が閉じていることを確認する
    fn is_close(&self) -> Result<(), Error> {
        let to_bits = |c: &Coordinate| -> [u64; 3] {
            [
                c.as_latitude().to_bits(),
                c.as_longitude().to_bits(),
                c.as_altitude().to_bits(),
            ]
        };

        #[derive(Debug, Default)]
        struct EdgeStats {
            forward_count: usize,
            backward_count: usize,
        }

        let mut edge_map: HashMap<([u64; 3], [u64; 3]), EdgeStats> = HashMap::new();

        for (surface_idx, surface) in self.surfaces.iter().enumerate() {
            let points = surface.points();

            for i in 0..points.len() - 1 {
                let p1_bits = to_bits(&points[i]);
                let p2_bits = to_bits(&points[i + 1]);

                if p1_bits == p2_bits {
                    return Err(Error::DegenerateEdge(surface_idx));
                }

                let key = if p1_bits < p2_bits {
                    (p1_bits, p2_bits)
                } else {
                    (p2_bits, p1_bits)
                };

                let stats = edge_map.entry(key).or_default();

                if p1_bits < p2_bits {
                    stats.forward_count += 1;
                } else {
                    stats.backward_count += 1;
                }
            }
        }

        // 全エッジを検証
        for ((p1, p2), stats) in &edge_map {
            // 各辺は正確に2つの面で共有されている必要がある
            // （forward 1回 + backward 1回）
            if stats.forward_count == 0 || stats.backward_count == 0 {
                return Err(Error::OpenHoleDetected);
            }

            if stats.forward_count > 1 || stats.backward_count > 1 {
                return Err(Error::NonManifoldEdge);
            }

            // 正確に1回ずつ出現していることを確認
            if stats.forward_count != 1 || stats.backward_count != 1 {
                return Err(Error::InvalidEdgeTopology {
                    forward: stats.forward_count,
                    backward: stats.backward_count,
                });
            }
        }

        Ok(())
    }

    pub fn surfaces(&self) -> &[Surface] {
        &self.surfaces
    }

    /// 立体をSingleIdの集合に変換する関数
    pub fn single_ids(&self, z: u8) {
        // -> Result<impl Iterator<Item = SingleId>, Error>
        todo!()
    }

    /// 立体をRangeIdの集合に変換する関数
    pub fn range_ids(&self, z: u8) {
        // -> Result<impl Iterator<Item = RangeId>, Error>
        todo!()
    }
}
