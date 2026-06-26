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
        );
        let coord = crate::Coordinate::try_from(ecef).unwrap();
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
        );
        let coord = crate::Coordinate::try_from(ecef).unwrap();
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
        );
        let coord = crate::Coordinate::try_from(ecef).unwrap();
        insta::assert_debug_snapshot!(coord);
    }

    /// 赤道付近で X/Y のわずかな差が OS 間で出やすい入力。
    #[ignore]
    #[test]
    fn equatorial_case_snapshot() {
        let ecef = Ecef::new(6_378_137.0, f64::EPSILON, 0.0);
        let coord = crate::Coordinate::try_from(ecef).unwrap();
        insta::assert_debug_snapshot!(coord);
    }
}
