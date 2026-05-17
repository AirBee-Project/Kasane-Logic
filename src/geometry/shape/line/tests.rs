use crate::{Coordinate, CoverSingleIds, Line};

fn sorted_ids(line: &Line, z: u8) -> Vec<String> {
    let mut ids: Vec<String> = line
        .cover_single_ids(z)
        .unwrap()
        .map(|id| id.to_string())
        .collect();
    ids.sort();
    ids
}

mod cover_single_ids {
    use super::*;

    /// 緯度・経度方向に伸びる水平な線分を変換する
    #[test]
    fn flat_segment_at_z25() {
        let p0 = Coordinate::new(35.681, 139.766, 0.0).unwrap();
        let p1 = Coordinate::new(35.682, 139.767, 0.0).unwrap();
        let line = Line::new([p0, p1]);
        insta::assert_debug_snapshot!(sorted_ids(&line, 25));
    }

    /// 高度方向に伸びる垂直な線分を変換する
    #[test]
    fn vertical_segment_at_z25() {
        let p0 = Coordinate::new(35.681, 139.766, 0.0).unwrap();
        let p1 = Coordinate::new(35.681, 139.766, 100.0).unwrap();
        let line = Line::new([p0, p1]);
        insta::assert_debug_snapshot!(sorted_ids(&line, 25));
    }

    /// 緯度・経度・高度がすべて異なる斜め線分を変換する
    #[test]
    fn diagonal_segment_at_z25() {
        let p0 = Coordinate::new(35.680, 139.765, 0.0).unwrap();
        let p1 = Coordinate::new(35.683, 139.768, 60.0).unwrap();
        let line = Line::new([p0, p1]);
        insta::assert_debug_snapshot!(sorted_ids(&line, 25));
    }
}
