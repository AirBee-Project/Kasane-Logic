use hashbrown::HashSet;

use crate::{
    Coordinate, Error, ExpandTriangles, Polygon, Shape, SingleId, geometry::traits::CoverSingleIds,
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
    /// # use spatial_id::geometry::{Coordinate, Polygon, Shape};
    /// let p0 = Coordinate::new(35.0, 139.0, 10.0).unwrap();
    /// let p1 = Coordinate::new(35.0, 139.001, 10.0).unwrap();
    /// let p2 = Coordinate::new(35.001, 139.0, 10.0).unwrap();
    /// let polygon = Polygon::new(vec![p0, p1, p2]);
    ///
    /// let center = polygon.center();
    /// assert!(center.latitude() > 35.0);
    /// ```
    fn center(&self) -> Coordinate {
        Coordinate::centroid(self.vertices.clone())
    }
}

impl CoverSingleIds for Polygon {
    type Value = ();

    /// ポリゴンが覆う空間IDを取得する。
    ///
    /// 指定されたズームレベルにおける、ポリゴンが覆うすべての空間IDを計算する。
    ///
    /// # 引数
    ///
    /// * `z` - ズームレベル
    ///
    /// # 戻り値
    ///
    /// 空間IDのイテレータ
    ///
    /// # 動作例
    ///
    /// タイトル: ポリゴンが覆う空間IDを取得
    /// ```
    /// # use spatial_id::geometry::{Coordinate, Polygon};
    /// # use spatial_id::geometry::CoverSingleIds;
    /// let p0 = Coordinate::new(35.0, 139.0, 10.0).unwrap();
    /// let p1 = Coordinate::new(35.0003, 139.0, 10.0).unwrap();
    /// let p2 = Coordinate::new(35.0, 139.0003, 10.0).unwrap();
    /// let p3 = Coordinate::new(35.0002, 139.0002, 10.3).unwrap();
    /// let polygon = Polygon::new(vec![p0, p1, p2, p3]);
    ///
    /// let ids: Vec<_> = polygon.cover_single_ids(20).unwrap().collect();
    /// assert!(ids.len() > 0);
    /// ```
    fn cover_single_ids_with<'a, V, F>(
        &'a self,
        z: impl Into<u8>,
        mut f: F,
    ) -> Result<impl Iterator<Item = (SingleId, V)> + 'a, Error>
    where
        V: Clone + 'a,
        F: FnMut(&'a Self::Value) -> V + 'a,
    {
        let value = f(&());
        let mut unique_ids = HashSet::new();
        let z: u8 = z.into();
        for triangle in self.expand_triangles() {
            let ids_iter = triangle.cover_single_ids(z)?;
            for id in ids_iter {
                unique_ids.insert(id);
            }
        }

        Ok(unique_ids.into_iter().map(move |id| (id, value.clone())))
    }
}
