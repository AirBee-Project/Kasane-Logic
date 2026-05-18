use crate::Coordinate;

#[cfg(not(feature = "temporal_id"))]
mod single_id {
    use super::*;

    #[test]
    /// 緯度がメルカトル投影の境界付近にあるため、OS によって floor の結果がぶれやすい例。
    fn boundary_latitude_at_z5() {
        let coord = Coordinate::new(40.97989806962012, 0.0, 0.0).unwrap();
        insta::assert_debug_snapshot!(coord.single_id(5).unwrap());
    }

    #[test]
    /// 高緯度・高ズームの組み合わせで、投影の最終桁の誤差が出やすい例。
    fn near_northern_limit_at_z25() {
        let coord = Coordinate::new(85.05109999999999, 179.99999999999997, 0.0).unwrap();
        insta::assert_debug_snapshot!(coord.single_id(25).unwrap());
    }

    #[test]
    /// 高度の floor がちょうど整数境界に近く、OS 差を拾いやすい例。
    fn altitude_boundary_at_z25() {
        let coord = Coordinate::new(35.681382, 139.76608399999998, 0.9999999999999999).unwrap();
        insta::assert_debug_snapshot!(coord.single_id(25).unwrap());
    }
}

mod round_trip {
    use super::*;

    #[test]
    /// Coordinate から Ecef へ変換する際の丸め誤差を確認する例。
    fn coordinate_to_ecef_snapshot() {
        let coord = Coordinate::new(43.068564, 41.3507138, 30.0).unwrap();
        let ecef: crate::Ecef = coord.into();
        insta::assert_debug_snapshot!(ecef);
    }

    #[test]
    /// 別の都市座標でも同様に、OS ごとの差を拾えるか確認する例。
    fn coordinate_to_ecef_snapshot_tokyo() {
        let coord = Coordinate::new(35.681382, 139.76608399999998, 10.0).unwrap();
        let ecef: crate::Ecef = coord.into();
        insta::assert_debug_snapshot!(ecef);
    }
}
