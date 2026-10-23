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
        let polygon = Polygon::new(vec![p0, p1, p2, p3]);
        insta::assert_debug_snapshot!(sorted_ids(&polygon, 18));
    }

    /// 水平面上の三角形ポリゴンを変換する
    #[test]
    fn horizontal_triangle_polygon_at_z18() {
        let p0 = Coordinate::new(35.681, 139.766, 10.0).unwrap();
        let p1 = Coordinate::new(35.684, 139.766, 10.0).unwrap();
        let p2 = Coordinate::new(35.682, 139.769, 10.0).unwrap();
        let polygon = Polygon::new(vec![p0, p1, p2]);
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
        let polygon = Polygon::new(vec![p0, p1, p2, p3, p4]);
        insta::assert_debug_snapshot!(sorted_ids(&polygon, 18));
    }

    /// 穴あきポリゴンの変換で内部がくり抜かれる
    #[test]
    fn polygon_with_hole() {
        let p0 = Coordinate::new(35.680, 139.760, 10.0).unwrap();
        let p1 = Coordinate::new(35.690, 139.760, 10.0).unwrap();
        let p2 = Coordinate::new(35.690, 139.770, 10.0).unwrap();
        let p3 = Coordinate::new(35.680, 139.770, 10.0).unwrap();
        let solid_poly = Polygon::new(vec![p0, p1, p2, p3]);

        let h0 = Coordinate::new(35.683, 139.763, 10.0).unwrap();
        let h1 = Coordinate::new(35.687, 139.763, 10.0).unwrap();
        let h2 = Coordinate::new(35.687, 139.767, 10.0).unwrap();
        let h3 = Coordinate::new(35.683, 139.767, 10.0).unwrap();
        let holed_poly = Polygon::with_holes(vec![p0, p1, p2, p3], vec![vec![h0, h1, h2, h3]]);

        let solid_ids: hashbrown::HashSet<_> = solid_poly.cover_single_ids(18u8).unwrap().collect();
        let holed_ids: hashbrown::HashSet<_> = holed_poly.cover_single_ids(18u8).unwrap().collect();

        assert!(holed_ids.len() < solid_ids.len());
        assert!(holed_ids.is_subset(&solid_ids));
    }

    #[test]
    fn polygon_with_epsilon() {
        use crate::Epsilon;
        let p0 = Coordinate::new(35.0, 139.0, 0.0).unwrap();
        let p1 = Coordinate::new(35.0, 139.001, 0.0).unwrap();
        let p2 = Coordinate::new(35.001, 139.0, 0.0).unwrap();
        let p_close = Coordinate::new(35.001000001, 139.0, 0.0).unwrap();

        // 許容誤差大: p_close は p2 と同一視されて縮退
        let poly = Polygon::with_epsilon(
            vec![p0, p1, p2, p_close],
            Epsilon::from_meters(1.0).unwrap(),
        );
        assert_eq!(poly.vertices().len(), 3);

        // 許容誤差極小: p_close は独立した頂点
        let poly_strict = Polygon::with_epsilon(
            vec![p0, p1, p2, p_close],
            Epsilon::from_meters(1e-6).unwrap(),
        );
        assert_eq!(poly_strict.vertices().len(), 4);
    }
}
