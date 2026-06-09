use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::{Coordinate, CoverSingleIds, Tube};

fn sorted_ids(tube: &Tube, z: u8) -> Vec<String> {
    let mut ids: Vec<String> = tube
        .cover_single_ids(z)
        .unwrap()
        .map(|id| id.to_string())
        .collect();
    ids.sort();
    ids
}

mod cover_single_ids {
    use super::*;

    /// 折れ曲がった2セグメントのチューブ（半径5m）を変換する
    #[test]
    fn bent_two_segment_tube_at_z18() {
        let p0 = Coordinate::new(35.681, 139.766, 10.0).unwrap();
        let p1 = Coordinate::new(35.682, 139.767, 10.0).unwrap();
        let p2 = Coordinate::new(35.683, 139.766, 10.0).unwrap();
        let tube = Tube::new(vec![p0, p1, p2], 5.0).unwrap();
        insta::assert_debug_snapshot!(sorted_ids(&tube, 18));
    }

    /// 高度方向に折れ曲がったチューブ（半径5m）を変換する
    #[test]
    fn vertically_bent_tube_at_z18() {
        let p0 = Coordinate::new(35.681, 139.766, 0.0).unwrap();
        let p1 = Coordinate::new(35.682, 139.766, 0.0).unwrap();
        let p2 = Coordinate::new(35.682, 139.766, 32.0).unwrap();
        let tube = Tube::new(vec![p0, p1, p2], 5.0).unwrap();
        insta::assert_debug_snapshot!(sorted_ids(&tube, 18));
    }

    /// 3セグメントの折れ曲がりチューブ（半径5m）を変換する
    #[test]
    fn three_segment_tube_at_z18() {
        let p0 = Coordinate::new(35.681, 139.766, 10.0).unwrap();
        let p1 = Coordinate::new(35.682, 139.767, 10.0).unwrap();
        let p2 = Coordinate::new(35.683, 139.767, 10.0).unwrap();
        let p3 = Coordinate::new(35.683, 139.766, 10.0).unwrap();
        let tube = Tube::new(vec![p0, p1, p2, p3], 5.0).unwrap();
        insta::assert_debug_snapshot!(sorted_ids(&tube, 18));
    }
}
