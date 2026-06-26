use crate::spatial_id::flex_id::FlexId;
use crate::spatial_id::range_id::RangeId;
use crate::spatial_id::single_id::SingleId;
use crate::spatial_id::traits::SpatialId;

#[test]
fn single_id_z30_no_overflow() {
    let max_x = (1 << 30) - 1;
    // z=30, f=0, x=max_x, y=max_x/2
    let mut id = SingleId::new(30, 0, max_x, max_x / 2).unwrap();
    // The following line will panic if the addition overflows an i32 (which was the bug)
    id.move_x(100);
    // Ensure distance calculations don't panic due to i32::pow
    let _ = id.length_x_meters();
    let _ = id.length_y_meters();
}

#[test]
fn flex_id_z30_no_overflow() {
    let max_x = (1 << 30) - 1;
    // f_z=30, f_i=0, x_z=30, x_i=max_x, y_z=30, y_i=max_x/2
    let mut id = FlexId::new(30, 0, 30, max_x, 30, max_x / 2).unwrap();
    id.move_x(100);
    let _ = id.length_x_meters();
    let _ = id.length_y_meters();
}

#[test]
fn range_id_z30_no_overflow() {
    let max_x = (1 << 30) - 1;
    // z=30, f=[0, 0], x=[max_x, max_x], y=[max_x/2, max_x/2]
    let mut id = RangeId::new(30, [0, 0], [max_x, max_x], [max_x / 2, max_x / 2]).unwrap();
    id.move_x(100);
    let _ = id.length_x_meters();
    let _ = id.length_y_meters();
}
