use crate::Ecef;

mod ecef_to_coordinate {
    use super::*;

    #[test]
    /// OSによって差が出てしまう計算の例。
    fn base_case_snapshot() {
        let ecef = Ecef::new(3503254.6369501497, 3083182.6924748584, 4333089.862951963);
        let coord = crate::Coordinate::try_from(ecef).unwrap();
        insta::assert_debug_snapshot!(coord);
    }

    #[test]
    /// 別の入力でも Bowring 反復の微小な誤差を拾うための例。
    fn west_coast_case_snapshot() {
        let ecef = Ecef::new(-2514383.4991819647, -4660897.97314988, 3567065.1289478777);
        let coord = crate::Coordinate::try_from(ecef).unwrap();
        insta::assert_debug_snapshot!(coord);
    }

    #[test]
    /// 地球中心からの距離が大きい入力でも、OS 差による末尾の揺れを確認する例。
    fn high_altitude_case_snapshot() {
        let ecef = Ecef::new(4075576.132840001, 3073465.9458809997, 4862865.221340001);
        let coord = crate::Coordinate::try_from(ecef).unwrap();
        insta::assert_debug_snapshot!(coord);
    }
}
