pub mod ops;

#[cfg(test)]
mod encode;

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
    fn spatial_siblings_are_returned_for_non_root_ids() {
        let id = SingleId::new(3, 3, 2, 7).unwrap();

        let siblings = id.spatial_siblings().unwrap();

        assert_eq!(siblings.len(), 7);
        assert!(!siblings.contains(&id));

        let parent = id.spatial_parent_at_zoom(2).unwrap();
        assert!(
            siblings
                .iter()
                .all(|s| s.spatial_parent_at_zoom(2).unwrap() == parent)
        );
    }

    #[test]
    fn spatial_siblings_is_none_at_root() {
        let id = SingleId::new(0, 0, 0, 0).unwrap();

        assert!(id.spatial_siblings().is_none());
    }

    #[test]
    fn spatial_parents_are_returned_from_nearest_to_root() {
        let id = SingleId::new(3, 3, 2, 7).unwrap();

        let parents: Vec<_> = id.spatial_parents().collect();

        assert_eq!(parents.len(), 3);
        assert_eq!(parents[0], SingleId::new(2, 1, 1, 3).unwrap());
        assert_eq!(parents[1], SingleId::new(1, 0, 0, 1).unwrap());
        assert_eq!(parents[2], SingleId::new(0, 0, 0, 0).unwrap());
    }

    #[test]
    fn spatial_parents_is_empty_at_root() {
        let id = SingleId::new(0, 0, 0, 0).unwrap();

        let parents: Vec<_> = id.spatial_parents().collect();

        assert!(parents.is_empty());
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
        let neighbors: Vec<_> = id.neighbors_share_face().collect();

        assert_eq!(neighbors.len(), 6);
        assert!(neighbors.contains(&SingleId::new(4, 7, 9, 10).unwrap()));
        assert!(neighbors.contains(&SingleId::new(4, 5, 9, 10).unwrap()));
        assert!(neighbors.contains(&SingleId::new(4, 6, 10, 10).unwrap()));
        assert!(neighbors.contains(&SingleId::new(4, 6, 8, 10).unwrap()));
        assert!(neighbors.contains(&SingleId::new(4, 6, 9, 11).unwrap()));
        assert!(neighbors.contains(&SingleId::new(4, 6, 9, 9).unwrap()));
    }

    #[test]
    fn twelve_edge_neighbors_are_generated() {
        let id = SingleId::new(4, 6, 9, 10).unwrap();
        let neighbors: Vec<_> = id.neighbors_share_edge().collect();

        assert_eq!(neighbors.len(), 12);
        // f-x 平面
        assert!(neighbors.contains(&SingleId::new(4, 7, 10, 10).unwrap()));
        assert!(neighbors.contains(&SingleId::new(4, 7, 8, 10).unwrap()));
        assert!(neighbors.contains(&SingleId::new(4, 5, 10, 10).unwrap()));
        assert!(neighbors.contains(&SingleId::new(4, 5, 8, 10).unwrap()));
        // f-y 平面
        assert!(neighbors.contains(&SingleId::new(4, 7, 9, 11).unwrap()));
        assert!(neighbors.contains(&SingleId::new(4, 7, 9, 9).unwrap()));
        assert!(neighbors.contains(&SingleId::new(4, 5, 9, 11).unwrap()));
        assert!(neighbors.contains(&SingleId::new(4, 5, 9, 9).unwrap()));
        // x-y 平面
        assert!(neighbors.contains(&SingleId::new(4, 6, 10, 11).unwrap()));
        assert!(neighbors.contains(&SingleId::new(4, 6, 10, 9).unwrap()));
        assert!(neighbors.contains(&SingleId::new(4, 6, 8, 11).unwrap()));
        assert!(neighbors.contains(&SingleId::new(4, 6, 8, 9).unwrap()));
    }

    #[test]
    fn eight_vertex_neighbors_are_generated() {
        let id = SingleId::new(4, 6, 9, 10).unwrap();
        let neighbors: Vec<_> = id.neighbors_share_vertex().collect();

        assert_eq!(neighbors.len(), 8);
        assert!(neighbors.contains(&SingleId::new(4, 7, 10, 11).unwrap()));
        assert!(neighbors.contains(&SingleId::new(4, 7, 10, 9).unwrap()));
        assert!(neighbors.contains(&SingleId::new(4, 7, 8, 11).unwrap()));
        assert!(neighbors.contains(&SingleId::new(4, 7, 8, 9).unwrap()));
        assert!(neighbors.contains(&SingleId::new(4, 5, 10, 11).unwrap()));
        assert!(neighbors.contains(&SingleId::new(4, 5, 10, 9).unwrap()));
        assert!(neighbors.contains(&SingleId::new(4, 5, 8, 11).unwrap()));
        assert!(neighbors.contains(&SingleId::new(4, 5, 8, 9).unwrap()));
    }

    #[test]
    fn twenty_six_neighbors_are_generated() {
        let id = SingleId::new(4, 6, 9, 10).unwrap();
        let neighbors: Vec<_> = id.neighbors_all().collect();

        assert_eq!(neighbors.len(), 26);
        assert!(neighbors.contains(&SingleId::new(4, 7, 10, 11).unwrap()));
        assert!(neighbors.contains(&SingleId::new(4, 5, 8, 9).unwrap()));
        assert!(neighbors.contains(&SingleId::new(4, 6, 10, 11).unwrap()));
        assert!(neighbors.contains(&SingleId::new(4, 6, 8, 9).unwrap()));
    }

    #[test]
    fn intersection_works() {
        let id1 = SingleId::new(2, 1, 1, 1).unwrap();
        // child is inside id1
        let child = SingleId::new(3, 2, 2, 3).unwrap();

        assert_eq!(id1.intersection(&child).unwrap(), child);
        assert_eq!(child.intersection(&id1).unwrap(), child);

        // Disjoint child
        let disjoint = SingleId::new(3, 4, 2, 3).unwrap(); // Outside f range
        assert!(id1.intersection(&disjoint).is_none());
    }

    #[test]
    fn difference_works() {
        let parent = SingleId::new(1, 0, 0, 0).unwrap();
        // Covers F:0, X:0, Y:0 at Z:1 (which covers Z:2 F:0..=1, X:0..=1, Y:0..=1)

        let child = SingleId::new(2, 0, 1, 1).unwrap();

        let diff: Vec<_> = parent.difference(&child).collect();
        // At Z=1 -> Z=2, parent is split into 8 children. One of them is `child`.
        // So difference should yield 7 SingleIds at Z=2.
        assert_eq!(diff.len(), 7);
        for d in &diff {
            assert_eq!(d.z(), 2);
            assert_ne!(d, &child);
        }
    }
}
