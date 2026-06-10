use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::{Coordinate, CoverSingleIds, Polygon};

fn sorted_ids(polygon: &Polygon, z: u8) -> Vec<String> {
    let mut ids: Vec<String> = polygon
        .cover_single_ids(z)
        .unwrap()
        .map(|id| id.to_string())
        .collect();
    ids.sort();
    ids
}

mod cover_single_ids {
    use super::*;

    /// 水平面上の四角形ポリゴンを変換する
    #[test]
    fn horizontal_quadrilateral_at_z18() {
        let p0 = Coordinate::new(35.681, 139.766, 10.0).unwrap();
        let p1 = Coordinate::new(35.684, 139.766, 10.0).unwrap();
        let p2 = Coordinate::new(35.684, 139.769, 10.0).unwrap();
        let p3 = Coordinate::new(35.681, 139.769, 10.0).unwrap();
        let polygon = Polygon::new(vec![p0, p1, p2, p3], 0.01);
        insta::assert_debug_snapshot!(sorted_ids(&polygon, 18));
    }

    /// 水平面上の三角形ポリゴンを変換する
    #[test]
    fn horizontal_triangle_polygon_at_z18() {
        let p0 = Coordinate::new(35.681, 139.766, 10.0).unwrap();
        let p1 = Coordinate::new(35.684, 139.766, 10.0).unwrap();
        let p2 = Coordinate::new(35.682, 139.769, 10.0).unwrap();
        let polygon = Polygon::new(vec![p0, p1, p2], 0.01);
        insta::assert_debug_snapshot!(sorted_ids(&polygon, 18));
    }

    /// 頂点数が多い多角形ポリゴンを変換する
    #[test]
    fn pentagon_at_z18() {
        let p0 = Coordinate::new(35.682, 139.767, 10.0).unwrap();
        let p1 = Coordinate::new(35.683, 139.766, 10.0).unwrap();
        let p2 = Coordinate::new(35.684, 139.767, 10.0).unwrap();
        let p3 = Coordinate::new(35.684, 139.769, 10.0).unwrap();
        let p4 = Coordinate::new(35.681, 139.769, 10.0).unwrap();
        let polygon = Polygon::new(vec![p0, p1, p2, p3, p4], 0.01);
        insta::assert_debug_snapshot!(sorted_ids(&polygon, 18));
    }
}
