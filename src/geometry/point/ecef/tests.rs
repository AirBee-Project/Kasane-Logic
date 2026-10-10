use crate::Ecef;

mod ecef_to_coordinate {
    use super::*;

    /// OSによって差が出てしまう計算の例。
    #[ignore]
    #[test]
    fn base_case_snapshot() {
        let ecef = Ecef::new(
            3_503_254.636_950_15,
            3_083_182.692_474_858,
            4_333_089.862_951_96,
        )
        .unwrap();
        let coord = crate::Coordinate::from(ecef);
        insta::assert_debug_snapshot!(coord);
    }

    /// 別の入力でも Bowring 反復の微小な誤差を拾うための例。
    #[ignore]
    #[test]
    fn west_coast_case_snapshot() {
        let ecef = Ecef::new(
            -2_514_383.499_181_965,
            -4_660_897.973_149_88,
            3_567_065.128_947_878,
        )
        .unwrap();
        let coord = crate::Coordinate::from(ecef);
        insta::assert_debug_snapshot!(coord);
    }

    /// 地球中心からの距離が大きい入力でも、OS 差による末尾の揺れを確認する例。
    #[ignore]
    #[test]
    fn high_altitude_case_snapshot() {
        let ecef = Ecef::new(
            4_075_576.132_840_001,
            3_073_465.945_881,
            4_862_865.221_340_001,
        )
        .unwrap();
        let coord = crate::Coordinate::from(ecef);
        insta::assert_debug_snapshot!(coord);
    }

    /// 赤道付近で X/Y のわずかな差が OS 間で出やすい入力。
    #[ignore]
    #[test]
    fn equatorial_case_snapshot() {
        let ecef = Ecef::new(6_378_137.0, f64::EPSILON, 0.0).unwrap();
        let coord = crate::Coordinate::from(ecef);
        insta::assert_debug_snapshot!(coord);
    }
}

/// 範囲の境界にある地理座標を ECEF に変換して戻しても、範囲外にならないことを確認する。
mod boundary_round_trip {
    use crate::{Coordinate, Ecef, Vec3, Vec3Ecef};

    const LATITUDES: [f64; 5] = [85.0511, -85.0511, 0.0, 35.0, -60.0];
    const ALTITUDES: [f64; 4] = [33_554_432.0, 0.0, -10_000.0, 1_000_000.0];

    fn boundary_coordinates() -> impl Iterator<Item = Coordinate> {
        LATITUDES.into_iter().flat_map(|lat| {
            ALTITUDES.into_iter().flat_map(move |alt| {
                (-180..=180)
                    .step_by(15)
                    .map(move |lon| Coordinate::new(lat, lon as f64, alt).unwrap())
            })
        })
    }

    #[test]
    fn vec3_ecef_to_coordinate_succeeds() {
        for coord in boundary_coordinates() {
            let vec: Vec3Ecef = coord.into();
            assert!(Coordinate::try_from(vec).is_ok(), "{coord:?}");
        }
    }

    #[test]
    fn ecef_can_be_recreated_from_components() {
        for coord in boundary_coordinates() {
            let ecef: Ecef = coord.into();
            assert!(Ecef::new(ecef.x(), ecef.y(), ecef.z()).is_ok(), "{coord:?}");
        }
    }

    #[test]
    fn ecef_to_coordinate_stays_in_range() {
        for coord in boundary_coordinates() {
            let ecef: Ecef = coord.into();
            let back = Coordinate::from(ecef);
            assert!(
                Coordinate::new(back.latitude(), back.longitude(), back.altitude()).is_ok(),
                "{coord:?} -> {back:?}"
            );
        }
    }

    #[test]
    fn far_out_of_range_is_still_rejected() {
        // 許容幅（1 cm）を大きく超える高度はエラーのまま
        let coord_vec: Vec3Ecef = Coordinate::new(35.0, 135.0, 33_554_432.0).unwrap().into();
        let scaled = coord_vec.scale(1.001);
        assert!(Coordinate::try_from(scaled).is_err());
    }
}
