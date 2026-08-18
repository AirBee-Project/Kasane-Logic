use crate::{Error, RangeId, SpatialIdError};

#[test]
fn set_x_wrapped_coarsened_no_wrap() {
    let mut id = RangeId::new(4, 0, 0, 0).unwrap();
    id.set_x_wrapped_coarsened(8, 11, 5);
    assert_eq!(id.x(), [4, 5]);
}

/// 折り返しがズーム差分を挟んでも1本の `RangeId` で正確に表現できる場合。
#[test]
fn set_x_wrapped_coarsened_wrap_representable() {
    let mut id = RangeId::new(3, 0, 0, 0).unwrap();
    id.set_x_wrapped_coarsened(-1, 1, 3);
    assert_eq!(id.x(), [7, 1]);
}

/// 丸め込みによって折り返し表現が破綻するため、安全側に倒して全域になる場合。
#[test]
fn set_x_wrapped_coarsened_wrap_collapses_to_full_domain() {
    let mut id = RangeId::new(1, 0, 0, 0).unwrap();
    id.set_x_wrapped_coarsened(1, 4, 2);
    assert_eq!(id.x(), [0, 1]);
}

#[test]
fn set_y_coarsened_clamped_basic() {
    let mut id = RangeId::new(4, 0, 0, 0).unwrap();
    assert!(id.set_y_coarsened_clamped(8, 11, 5));
    assert_eq!(id.y(), [4, 5]);
}

/// 範囲の一部が有効範囲をはみ出す場合は切り詰められる。
#[test]
fn set_y_coarsened_clamped_clamps_out_of_range_part() {
    let mut id = RangeId::new(4, 0, 0, 0).unwrap();
    assert!(id.set_y_coarsened_clamped(-3, 11, 5));
    assert_eq!(id.y(), [0, 5]);
}

#[test]
fn set_f_coarsened_clamped_basic() {
    let mut id = RangeId::new(4, 0, 0, 0).unwrap();
    assert!(id.set_f_coarsened_clamped(8, 11, 5));
    assert_eq!(id.f(), [4, 5]);
}

#[test]
fn x_edges_shift_handles_self_already_wrapped_full_domain() {
    let mut id = RangeId::new(2, 0, 0, 0).unwrap();
    id.set_x([2, 1]).unwrap();
    assert_eq!(id.x(), [2, 1]);

    let result = id.x_edges_shift(2, -1, 1).unwrap().unwrap();
    assert_eq!(result.x(), [0, 3]);
}

#[test]
fn x_edges_shift_handles_self_already_wrapped_partial() {
    let mut id = RangeId::new(3, 0, 0, 0).unwrap();
    id.set_x([6, 1]).unwrap();
    assert_eq!(id.x(), [6, 1]);

    let result = id.x_edges_shift(3, -1, 1).unwrap().unwrap();
    assert_eq!(result.x(), [5, 2]);
}

/// `target_z` が `ZoomLevel::MAX` を超える場合はエラーになる（シフト量オーバーフローで
/// パニックしたり、リリースビルドで黙って誤った結果を返したりしてはいけない）。
#[test]
fn x_edges_shift_rejects_target_z_beyond_zoom_level_max() {
    let id = RangeId::new(2, 0, 0, 0).unwrap();
    let too_large = crate::ZoomLevel::MAX.get() + 1;

    let result = id.x_edges_shift(too_large, 0, 0);
    assert_eq!(
        result,
        Err(Error::SpatialId(SpatialIdError::ZOutOfRange { z: too_large }))
    );
}

/// `target_z == ZoomLevel::MAX` はちょうど境界なので、エラーにならない。
#[test]
fn x_edges_shift_accepts_target_z_at_zoom_level_max() {
    let id = RangeId::new(2, 0, 0, 0).unwrap();
    let max = crate::ZoomLevel::MAX.get();

    assert!(id.x_edges_shift(max, 0, 0).is_ok());
}
