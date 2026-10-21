use alloc::vec::Vec;

use crate::RangeId;

#[test]
fn shift_range_x_no_wrap() {
    let id = RangeId::new(4, 0, [2, 2], 0).unwrap();
    let mut out = Vec::new();
    id.shift_range(5, [0, 0], [4, 5], [0, 0], &mut out);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].z(), 5);
    assert_eq!(out[0].x(), [8, 10]);
}

#[test]
fn shift_range_x_splits_wrapped_into_ascending_ranges() {
    let id = RangeId::new(3, 0, [0, 0], 0).unwrap();
    let mut out = Vec::new();
    id.shift_range(3, [0, 0], [-1, 1], [0, 0], &mut out);
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].x(), [7, 7]);
    assert_eq!(out[1].x(), [0, 1]);
}

#[test]
fn shift_range_x_handles_self_already_wrapped_full_domain() {
    let mut id = RangeId::new(2, 0, 0, 0).unwrap();
    id.set_x([2, 1]).unwrap();
    let mut out = Vec::new();
    id.shift_range(2, [0, 0], [-1, 1], [0, 0], &mut out);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].x(), [0, 3]);
}

#[test]
fn shift_range_x_handles_self_already_wrapped_partial() {
    let mut id = RangeId::new(3, 0, 0, 0).unwrap();
    id.set_x([6, 1]).unwrap();
    let mut out = Vec::new();
    id.shift_range(3, [0, 0], [-1, 1], [0, 0], &mut out);
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].x(), [5, 7]);
    assert_eq!(out[1].x(), [0, 2]);
}

#[test]
fn shift_range_f_and_y_clamps_and_promotes_zoom() {
    let id = RangeId::new(4, [2, 2], 0, [0, 1]).unwrap();
    let mut out = Vec::new();
    id.shift_range(5, [4, 5], [0, 0], [-3, 1], &mut out);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].z(), 5);
    assert_eq!(out[0].f(), [8, 10]);
    assert_eq!(out[0].y(), [0, 4]);
}

#[test]
fn shift_range_completely_out_of_bounds_is_empty() {
    let id = RangeId::new(3, [0, 1], 0, [0, 1]).unwrap();
    let mut out = Vec::new();
    id.shift_range(3, [-20, -20], [0, 0], [0, 0], &mut out);
    assert!(out.is_empty());

    id.shift_range(3, [0, 0], [0, 0], [-5, -5], &mut out);
    assert!(out.is_empty());
}

#[test]
fn spatial_parent_at_zoom_wrapped_x_collapses_to_full_domain() {
    let mut id = RangeId::new(3, 0, 0, 0).unwrap();
    id.set_x([3, 2]).unwrap();
    let parent = id.spatial_parent_at_zoom(2).unwrap();
    assert_eq!(parent.x(), [0, 3]);
}

#[test]
fn spatial_parent_at_zoom_wrapped_x_preserves_gap() {
    let mut id = RangeId::new(3, 0, 0, 0).unwrap();
    id.set_x([6, 1]).unwrap();
    let parent = id.spatial_parent_at_zoom(2).unwrap();
    assert_eq!(parent.x(), [3, 0]);
}
