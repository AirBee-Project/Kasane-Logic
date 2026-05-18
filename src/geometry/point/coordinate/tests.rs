use crate::Coordinate;

#[test]
/// OSによって差が出てしまう計算の例
fn test_coordinate_single_id_snapshot() {
    let coord = Coordinate::new(40.97989806962012, 135.0, 10.0).unwrap();
    let id = coord.single_id(5).unwrap();
    insta::assert_debug_snapshot!(id);
}
