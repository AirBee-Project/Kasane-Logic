use crate::Ecef;

#[test]
/// OSによって差が出てしまう計算の例
fn test_ecef_to_coordinate_snapshot() {
    let ecef = Ecef::new(3503254.6369501497, 3083182.6924748584, 4333089.862951963);
    let coord = crate::Coordinate::try_from(ecef).unwrap();
    insta::assert_debug_snapshot!(coord);
}
