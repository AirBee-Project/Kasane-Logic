use std::collections::btree_map::Range;

use kasane_logic::spatial_id::{
    SpatialId,
    constants::{F_MAX, F_MIN, MAX_ZOOM_LEVEL, XY_MAX},
    range::RangeId,
    segment::{Segment, encode::EncodeSegment},
};
use rand::Rng;

fn main() {
    let segment1: Vec<_> = Segment::<u32>::new(3, [5, 5]).collect();
    let segment2: Vec<_> = Segment::<u32>::new(3, [5, 5]).collect();

    let s1 = segment1.first().unwrap().clone();
    let s2 = segment2.first().unwrap().clone();

    let encode1 = EncodeSegment::from(s1);
    let encode2 = EncodeSegment::from(s2);

    println!("{:?}", encode1.relation(&encode2));
}
