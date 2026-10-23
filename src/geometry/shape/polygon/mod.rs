use alloc::vec::Vec;

use crate::{Coordinate, geometry::Tolerance};

pub mod geometry_relation;
pub mod impls;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
/// 3次元空間における多角形（ポリゴン）を表す型。
///
/// 頂点リスト（[Coordinate] のVec）によって定義される平面的な領域を表現する。
/// 生成時に頂点の重複排除などが行われ、幾何計算に適した状態に保たれる。
pub struct Polygon {
    vertices: Vec<Coordinate>,
}

impl Polygon {
    /// 頂点座標のリストからデフォルト許容誤差（[Tolerance::DEFAULT]）を使用して新しい [Polygon] を作成。
    ///
    /// # 処理内容
    /// - 連続して重複している頂点（デフォルト許容誤差以内の距離）を1つに統合。
    /// - 始点と終点が重複している場合（閉じたリング）、終点を削除して開いた頂点リストに正規化。
    /// - 処理によって点の数が3未満だった場合は`Vec<Coordinate>`を空にする。
    ///
    /// # 引数
    /// - `raw_points` - ポリゴンを構成する頂点のリスト。
    pub fn new(raw_points: Vec<Coordinate>) -> Self {
        Self::with_tolerance(raw_points, Tolerance::DEFAULT)
    }

    /// 頂点座標のリストと指定された許容誤差から新しい [Polygon] を作成。
    ///
    /// # 処理内容
    /// - 連続して重複している頂点（`tolerance` 以内の距離）を1つに統合。
    /// - 始点と終点が重複している場合（閉じたリング）、終点を削除して開いた頂点リストに正規化。
    /// - 処理によって点の数が3未満だった場合は`Vec<Coordinate>`を空にする。
    ///
    /// # 引数
    /// - `raw_points` - ポリゴンを構成する頂点のリスト。
    /// - `tolerance` - 同一点とみなす許容誤差。
    pub fn with_tolerance(raw_points: Vec<Coordinate>, tolerance: Tolerance) -> Self {
        if raw_points.is_empty() {
            return Self { vertices: vec![] };
        }

        let mut vertices: Vec<Coordinate> = Vec::new();

        for p in raw_points {
            if let Some(last) = vertices.last() {
                if !last.eq_tolerance(&p, tolerance) {
                    vertices.push(p);
                }
            } else {
                vertices.push(p);
            }
        }

        if vertices.len() > 2 && vertices[0].eq_tolerance(vertices.last().unwrap(), tolerance) {
            vertices.pop();
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
