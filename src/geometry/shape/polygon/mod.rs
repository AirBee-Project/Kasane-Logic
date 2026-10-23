use alloc::vec::Vec;

use crate::Coordinate;

pub mod geometry_relation;
pub mod impls;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
/// 3次元空間における多角形（ポリゴン）を表す型。
///
/// 頂点リスト（[Coordinate] のVec）および任意の穴（内周リング）によって定義される平面的な領域を表現する。
/// 生成時に頂点の重複排除などが行われ、幾何計算に適した状態に保たれる。
pub struct Polygon {
    vertices: Vec<Coordinate>,
    holes: Vec<Vec<Coordinate>>,
}

impl Polygon {
    /// 頂点座標のリストから穴のない新しい [Polygon] を作成する。
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
        Self::with_holes(raw_points, Vec::new(), epsilon)
    }

    /// 外周頂点リストおよび穴（内周リング）リストから新しい [Polygon] を作成する。
    pub fn with_holes(
        raw_points: Vec<Coordinate>,
        raw_holes: Vec<Vec<Coordinate>>,
        epsilon: f64,
    ) -> Self {
        let vertices = Self::clean_ring(raw_points, epsilon);
        if vertices.len() < 3 {
            return Self {
                vertices: Vec::new(),
                holes: Vec::new(),
            };
        }

        let holes = raw_holes
            .into_iter()
            .map(|ring| Self::clean_ring(ring, epsilon))
            .filter(|ring| ring.len() >= 3)
            .collect();

        Self { vertices, holes }
    }

    fn clean_ring(raw_points: Vec<Coordinate>, epsilon: f64) -> Vec<Coordinate> {
        if raw_points.is_empty() {
            return Vec::new();
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

        if vertices.len() > 2 && vertices[0].eq_epsilon(vertices.last().unwrap(), epsilon) {
            vertices.pop();
        }

        if vertices.len() < 3 {
            return Vec::new();
        }

        vertices
    }

    /// [Polygon]の外周を構成する点を返す。
    pub fn vertices(&self) -> &Vec<Coordinate> {
        &self.vertices
    }

    /// [Polygon]の穴（内周リング）を構成する点リストを返す。
    pub fn holes(&self) -> &[Vec<Coordinate>] {
        &self.holes
    }
}
