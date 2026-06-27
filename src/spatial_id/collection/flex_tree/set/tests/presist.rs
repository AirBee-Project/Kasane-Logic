#[cfg(all(test, feature = "persist"))]
mod persist_tests {
    use super::SpatialIdSet;
    use crate::{RangeId, SingleId};

    /// FlexId をそのまま比較する（独自 PartialEq の zoom 正規化展開は使わない）。
    fn sorted(set: &SpatialIdSet) -> Vec<crate::FlexId> {
        let mut v: Vec<_> = set.iter().collect();
        v.sort();
        v
    }

    #[test]
    fn round_trip() {
        let mut set = SpatialIdSet::new();
        set.insert(SingleId::new(20, 0, 0, 0).unwrap());
        set.insert(SingleId::new(18, 1, 5, 7).unwrap());
        set.insert(RangeId::new(5, [1, 4], [8, 9], [5, 6]).unwrap());

        let bytes = set.to_bytes().unwrap();
        let restored = unsafe { SpatialIdSet::from_bytes(&bytes).unwrap() };

        assert_eq!(sorted(&set), sorted(&restored));
        assert_eq!(set.count(), restored.count());
    }

    #[test]
    fn round_trip_empty() {
        let set = SpatialIdSet::new();
        let bytes = set.to_bytes().unwrap();
        let restored = unsafe { SpatialIdSet::from_bytes(&bytes).unwrap() };
        assert!(restored.is_empty());
    }
}
