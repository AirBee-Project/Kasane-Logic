use hashbrown::HashSet;

use crate::{
    Coordinate, ExpandTriangles, Polygon, Shape, SingleId, geometry::traits::CoverSingleIds,
};

impl Shape for Polygon {
    /// ポリゴンの重心を取得する。
    ///
    /// 構成する全ての頂点の座標の平均値を算出し、ポリゴンの重心座標として返す。
    ///
    /// # 動作例
    ///
    /// タイトル: ポリゴンの重心を計算
    /// ```
    /// # use kasane_logic::{Coordinate, Polygon, Shape};
    /// let p0 = Coordinate::new(35.0, 139.0, 10.0).unwrap();
    /// let p1 = Coordinate::new(35.0, 139.001, 10.0).unwrap();
    /// let p2 = Coordinate::new(35.001, 139.0, 10.0).unwrap();
    /// let polygon = Polygon::new(vec![p0, p1, p2], 0.01);
    ///
    /// let center = polygon.center();
    /// assert!(center.latitude() > 35.0);
    /// ```
    fn center(&self) -> Coordinate {
        Coordinate::center_gravity(self.vertices.clone())
    }
}

impl CoverSingleIds for Polygon {
    /// ポリゴン領域を覆う全ての [`SingleId`] を取得する。
    ///
    /// ポリゴンを内部で複数の三角形に分割し、それぞれの三角形が覆う空間IDの和集合を計算して返す。
    /// 境界上の空間IDは重複しないように排除される。
    ///
    /// # パラメーター
    /// * `z` — 取得する空間IDのズームレベル（0 以上 `MAX_ZOOM_LEVEL` 以下。現状 30）
    ///
    /// # バリデーション
    /// - `z` が `MAX_ZOOM_LEVEL` を超える場合、`crate::Error`（例: `Error::SpatialId(SpatialIdError::ZOutOfRange { .. })`）を返す。
    /// - 内部の三角形分割や各三角形の空間ID計算で不正な座標が現れた場合などは、対応する `crate::Error` を返す。
    ///
    /// # 動作コスト
    /// ポリゴンの頂点数による三角形分割の計算量、および分割された三角形の面積（取得する空間IDの密度）に比例して計算コストが増加する。
    /// 巨大なポリゴンや高いズームレベルを指定した場合は、イテレータの評価に極めて高いコストがかかる。
    ///
    /// # 動作例
    ///
    /// タイトル: ポリゴンから空間IDを取得
    /// ```
    /// # use kasane_logic::{Coordinate, Polygon};
    /// # use kasane_logic::geometry::traits::CoverSingleIds;
    /// let p0 = Coordinate::new(35.0, 139.0, 10.0).unwrap();
    /// let p1 = Coordinate::new(35.0003, 139.0, 10.0).unwrap();
    /// let p2 = Coordinate::new(35.0, 139.0003, 10.0).unwrap();
    /// let p3 = Coordinate::new(35.0002, 139.0002, 10.3).unwrap();
    /// let polygon = Polygon::new(vec![p0, p1, p2, p3], 0.01);
    ///
    /// let ids: Vec<_> = polygon.cover_single_ids(20).unwrap().collect();
    /// assert!(ids.len() > 0);
    /// ```
    fn cover_single_ids(&self, z: u8) -> Result<impl Iterator<Item = SingleId>, crate::Error> {
        let mut unique_ids = HashSet::new();

        for triangle in self.expand_triangles() {
            let ids_iter = triangle.cover_single_ids(z)?;
            for id in ids_iter {
                unique_ids.insert(id);
            }
        }

        Ok(unique_ids.into_iter())
    }
}
