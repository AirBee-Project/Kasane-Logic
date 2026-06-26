use crate::spatial_id::zoom_level::ZoomLevel;
use alloc::vec::Vec;

use crate::{SingleId, SpatialIdError, error::Error};

#[test]
fn spatial_encode_and_decode_roundtrip() {
    let cases = [
        SingleId::new(0, -1, 0, 0).unwrap(),
        SingleId::new(3, 3, 2, 7).unwrap(),
        SingleId::new(5, -12, 9, 10).unwrap(),
        SingleId::new(10, 511, 1023, 1023).unwrap(),
        SingleId::new(30, 1_073_741_823, 1_073_741_823, 1_073_741_823).unwrap(),
    ];

    for id in cases {
        let encoded = id.spatial_encode();
        let decoded = SingleId::spatial_decode(&encoded).unwrap();

        assert_eq!(decoded, id);
    }
}

#[test]
fn spatial_encode_prefix_orders_parent_before_children() {
    let parent = SingleId::new(3, 3, 2, 7).unwrap();
    let child = SingleId::new(4, 6, 4, 14).unwrap();

    let parent_min = parent.spatial_encode();
    let parent_max = parent.spatial_encode_prefix_max();
    let child_key = child.spatial_encode();

    assert!(parent_min <= child_key);
    assert!(child_key <= parent_max);
}

#[test]
fn spatial_encode_prefix_max_is_not_smaller_than_encode() {
    let id = SingleId::new(5, 3, 2, 10).unwrap();
    assert!(id.spatial_encode() <= id.spatial_encode_prefix_max());
}

#[test]
fn spatial_encode_prefix_max_stops_before_next_sibling_subtree() {
    let parent = SingleId::new(2, 1, 1, 1).unwrap();
    let children: Vec<_> = parent.spatial_children_at_zoom(3).unwrap().collect();

    let subtree_root = &children[0];
    let next_sibling_root = &children[1];
    let subtree_max = subtree_root.spatial_encode_prefix_max();

    assert!(subtree_root.spatial_encode() <= subtree_max);
    assert!(subtree_max < next_sibling_root.spatial_encode());
}

#[test]
fn spatial_encode_places_zoom_level_in_low_five_bits() {
    let id = SingleId::new(3, 3, 2, 7).unwrap();
    let bytes = id.spatial_encode();

    assert_eq!(bytes[11] & 0b0001_1111, 3);
}

#[test]
fn spatial_encode_roundtrip_at_zero_zoom_boundaries() {
    let cases = [
        SingleId::new(0, -1, 0, 0).unwrap(),
        SingleId::new(0, 0, 0, 0).unwrap(),
    ];

    for id in cases {
        let encoded = id.spatial_encode();
        let decoded = SingleId::spatial_decode(&encoded).unwrap();

        assert_eq!(decoded, id);
    }
}

#[test]
fn spatial_encode_roundtrip_at_max_zoom_boundaries() {
    let z = ZoomLevel::MAX.get();
    let cases = [
        SingleId::new(z, ZoomLevel::new(z as usize as u8).unwrap().f_min(), 0, 0).unwrap(),
        SingleId::new(
            z,
            ZoomLevel::new(z as usize as u8).unwrap().f_max(),
            ZoomLevel::new(z as usize as u8).unwrap().xy_max(),
            ZoomLevel::new(z as usize as u8).unwrap().xy_max(),
        )
        .unwrap(),
    ];

    for id in cases {
        let encoded = id.spatial_encode();
        let decoded = SingleId::spatial_decode(&encoded).unwrap();

        assert_eq!(decoded, id);
    }
}

#[test]
fn spatial_decode_rejects_invalid_zoom_bits() {
    let id = SingleId::new(3, 3, 2, 7).unwrap();
    let mut bytes = id.spatial_encode();

    bytes[11] = (bytes[11] & 0b1110_0000) | 0b0001_1111;

    let result = SingleId::spatial_decode(&bytes);

    assert!(matches!(
        result,
        Err(Error::SpatialId(SpatialIdError::ZOutOfRange { z: 31 }))
    ));
}
