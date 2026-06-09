use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::Coordinate;

#[cfg(not(feature = "temporal_id"))]
mod single_id {
    use super::*;

    /// 緯度がメルカトル投影の境界付近にあるため、OS によって floor の結果がぶれやすい例。
    #[ignore]
    #[test]
    fn boundary_latitude_at_z5() {
        let coord = Coordinate::new(40.979_898_069_620_12, 0.0, 0.0).unwrap();
        insta::assert_debug_snapshot!(coord.single_id(5).unwrap());
    }

    /// 高緯度・高ズームの組み合わせで、投影の最終桁の誤差が出やすい例。
    #[ignore]
    #[test]
    fn near_northern_limit_at_z25() {
        let coord = Coordinate::new(85.051_1, 180.0, 0.0).unwrap();
        insta::assert_debug_snapshot!(coord.single_id(25).unwrap());
    }

    /// 高度の floor がちょうど整数境界に近く、OS 差を拾いやすい例。
    #[ignore]
    #[test]
    fn altitude_boundary_at_z25() {
        let coord = Coordinate::new(35.681_382, 139.766_084, 1.0).unwrap();
        insta::assert_debug_snapshot!(coord.single_id(25).unwrap());
    }

    /// 緯度の境界をわずかにまたぐ入力で、OS ごとの差を拾いやすくする例。
    #[ignore]
    #[test]
    fn latitude_epsilon_boundary_at_z5() {
        let coord = Coordinate::new(40.979_898_069_620_12 + f64::EPSILON, 0.0, 0.0).unwrap();
        insta::assert_debug_snapshot!(coord.single_id(5).unwrap());
    }

    /// 経度の反対側境界付近で、floor の丸め差を確認する例。
    #[ignore]
    #[test]
    fn longitude_dateline_boundary_at_z25() {
        let coord = Coordinate::new(0.0, 180.0 - f64::EPSILON, 0.0).unwrap();
        insta::assert_debug_snapshot!(coord.single_id(25).unwrap());
    }
}

mod round_trip {
    use super::*;

    #[test]
    /// Coordinate から Ecef へ変換する際の丸め誤差を確認する例。
    fn coordinate_to_ecef_snapshot() {
        let coord = Coordinate::new(43.068_564, 41.350_713_8, 30.0).unwrap();
        let ecef: crate::Ecef = coord.into();
        insta::assert_debug_snapshot!(ecef);
    }

    #[test]
    /// 別の都市座標でも同様に、OS ごとの差を拾えるか確認する例。
    fn coordinate_to_ecef_snapshot_tokyo() {
        let coord = Coordinate::new(35.681_382, 139.766_084, 10.0).unwrap();
        let ecef: crate::Ecef = coord.into();
        insta::assert_debug_snapshot!(ecef);
    }

    #[test]
    /// 北緯の高い地点では三角関数の丸め誤差が OS 間で出やすい。
    fn coordinate_to_ecef_snapshot_high_latitude() {
        let coord = Coordinate::new(85.051_1 - f64::EPSILON, 45.0, 123.0).unwrap();
        let ecef: crate::Ecef = coord.into();
        insta::assert_debug_snapshot!(ecef);
    }
}
