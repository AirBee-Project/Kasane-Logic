#[allow(unused_imports)]
use alloc::boxed::Box;
#[allow(unused_imports)]
use alloc::rc::Rc;
#[allow(unused_imports)]
use alloc::string::{String, ToString};
#[allow(unused_imports)]
use alloc::vec::Vec;

use crate::{Coordinate, Ecef, FractionalId, geometry::traits::CoverSingleIds};

#[test]
fn test_fractional_id_to_single_id() {
    let fid = FractionalId::new(4, 5.5, 6.2, 7.8).unwrap();
    let sid = fid.single_id();
    assert_eq!(sid.z(), 4);
    assert_eq!(sid.f(), 5);
    assert_eq!(sid.x(), 6);
    assert_eq!(sid.y(), 7);
}

#[test]
fn test_fractional_id_to_coordinate() {
    let fid = FractionalId::new(4, 5.5, 6.2, 7.8).unwrap();
    let coord: Coordinate = fid.into();
    assert!(coord.latitude() >= -90.0 && coord.latitude() <= 90.0);
    assert!(coord.longitude() >= -180.0 && coord.longitude() <= 180.0);
}

#[test]
fn test_fractional_id_to_ecef() {
    let fid = FractionalId::new(4, 5.5, 6.2, 7.8).unwrap();
    let ecef: Ecef = fid.into();
    let origin = Ecef::new(0.0, 0.0, 0.0);
    let r = ecef.distance(&origin);
    assert!(r >= 0.0);
}

#[test]
fn test_cover_single_ids() {
    let fid = FractionalId::new(4, 5.5, 6.2, 7.8).unwrap();

    let mut iter = fid.cover_single_ids(4).unwrap();
    let sid = iter.next().unwrap();

    assert_eq!(sid.z(), 4);

    assert!(iter.next().is_none());
}
