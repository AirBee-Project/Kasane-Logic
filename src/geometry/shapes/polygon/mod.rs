use crate::{Coordinate, Ecef, Triangle};

pub mod geometry_relation;
pub mod impls;

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
}
