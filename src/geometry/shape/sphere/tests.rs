use crate::{Coordinate, CoverSingleIds, Sphere};

fn sorted_ids(sphere: &Sphere, z: u8) -> Vec<String> {
    let mut ids: Vec<String> = sphere
        .cover_single_ids(z)
        .unwrap()
        .map(|id| id.to_string())
        .collect();
    ids.sort();
    ids
}

mod cover_single_ids {
    use super::*;

    /// 小さな球体（半径30m）を変換する
    #[test]
    fn small_sphere_radius_30m_at_z20() {
        let center = Coordinate::new(35.681, 139.766, 10.0).unwrap();
        let sphere = Sphere::new(center, 30.0).unwrap();
        insta::assert_debug_snapshot!(sorted_ids(&sphere, 20));
    }

    /// 中程度の球体（半径100m）を変換する
    #[test]
    fn medium_sphere_radius_100m_at_z18() {
        let center = Coordinate::new(35.681, 139.766, 50.0).unwrap();
        let sphere = Sphere::new(center, 100.0).unwrap();
        insta::assert_debug_snapshot!(sorted_ids(&sphere, 18));
    }

    /// 地表付近のゼロ高度における球体を変換する
    #[test]
    fn sphere_at_ground_level_at_z20() {
        let center = Coordinate::new(35.681, 139.766, 0.0).unwrap();
        let sphere = Sphere::new(center, 30.0).unwrap();
        insta::assert_debug_snapshot!(sorted_ids(&sphere, 20));
    }
}
