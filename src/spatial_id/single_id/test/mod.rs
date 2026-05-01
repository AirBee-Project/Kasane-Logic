#[cfg(test)]
mod tests {
    use crate::{Error, SingleId, SpatialIdError};

    #[test]
    fn children_at_zoom_works() {
        let id = SingleId::new(3, 3, 2, 7).unwrap();

        let children: Vec<_> = id.spatial_children_at_zoom(4).unwrap().collect();

        assert_eq!(children.len(), 8);
        assert_eq!(children.first().unwrap().z(), 4);
        assert_eq!(children.first().unwrap().f(), 6);
        assert_eq!(children.first().unwrap().x(), 4);
        assert_eq!(children.first().unwrap().y(), 14);
    }

    #[test]
    fn parent_at_zoom_works() {
        let id = SingleId::new(4, 6, 9, 14).unwrap();

        let parent = id.spatial_parent_at_zoom(3).unwrap();

        assert_eq!(parent.z(), 3u8);
        assert_eq!(parent.f(), 3i32);
        assert_eq!(parent.x(), 4u32);
        assert_eq!(parent.y(), 7u32);
    }

    #[test]
    fn zoom_direction_mismatch_returns_error() {
        let id = SingleId::new(3, 3, 2, 7).unwrap();

        let result = id.spatial_children_at_zoom(2);

        assert!(matches!(
            result,
            Err(Error::SpatialId(
                SpatialIdError::ZoomLevelTransitionOutOfRange {
                    current_z: 3,
                    target_z: 2
                }
            ))
        ));
    }

    #[test]
    fn six_neighbors_are_generated() {
        let id = SingleId::new(4, 6, 9, 10).unwrap();
        let neighbors: Vec<_> = id.spatial_neighbors_6().collect();

        assert_eq!(neighbors.len(), 6);
        assert!(neighbors.contains(&SingleId::new(4, 7, 9, 10).unwrap()));
        assert!(neighbors.contains(&SingleId::new(4, 5, 9, 10).unwrap()));
        assert!(neighbors.contains(&SingleId::new(4, 6, 10, 10).unwrap()));
        assert!(neighbors.contains(&SingleId::new(4, 6, 8, 10).unwrap()));
        assert!(neighbors.contains(&SingleId::new(4, 6, 9, 11).unwrap()));
        assert!(neighbors.contains(&SingleId::new(4, 6, 9, 9).unwrap()));
    }

    #[test]
    fn twenty_six_neighbors_are_generated() {
        let id = SingleId::new(4, 6, 9, 10).unwrap();
        let neighbors: Vec<_> = id.spatial_neighbors_26().collect();

        assert_eq!(neighbors.len(), 26);
        assert!(neighbors.contains(&SingleId::new(4, 7, 10, 11).unwrap()));
        assert!(neighbors.contains(&SingleId::new(4, 5, 8, 9).unwrap()));
        assert!(neighbors.contains(&SingleId::new(4, 6, 10, 11).unwrap()));
        assert!(neighbors.contains(&SingleId::new(4, 6, 8, 9).unwrap()));
    }
}
