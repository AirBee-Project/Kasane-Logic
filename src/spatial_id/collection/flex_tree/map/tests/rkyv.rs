#[cfg(all(test, feature = "persist"))]
mod persist_tests {
    use crate::SpatialIdMap;
    use crate::spatial_id::collection::flex_tree::map::persist::ArchivedMap;
    use crate::{RangeId, SingleId};
    use alloc::vec::Vec;

    fn sorted(map: &SpatialIdMap<Vec<u8>>) -> Vec<(crate::FlexId, Vec<u8>)> {
        let mut v: Vec<_> = map.iter().map(|(f, val)| (f, val.clone())).collect();
        v.sort();
        v
    }

    #[test]
    fn round_trip() {
        let mut map = SpatialIdMap::<Vec<u8>>::new();
        map.insert(SingleId::new(20, 0, 0, 0).unwrap(), b"alpha".to_vec());
        map.insert(SingleId::new(20, 0, 2, 3).unwrap(), b"alpha".to_vec());
        map.insert(SingleId::new(18, 1, 5, 7).unwrap(), b"beta".to_vec());
        map.insert(
            RangeId::new(5, [1, 4], [8, 9], [5, 6]).unwrap(),
            b"gamma".to_vec(),
        );

        let bytes = map.to_bytes().unwrap();
        let restored = unsafe { SpatialIdMap::<Vec<u8>>::from_bytes(&bytes).unwrap() };

        assert_eq!(sorted(&map), sorted(&restored));
        assert_eq!(map.count(), restored.count());
    }

    #[test]
    fn round_trip_empty() {
        let map = SpatialIdMap::<Vec<u8>>::new();
        let bytes = map.to_bytes().unwrap();
        let restored = unsafe { SpatialIdMap::<Vec<u8>>::from_bytes(&bytes).unwrap() };
        assert!(restored.is_empty());
    }

    /// ZeroCopy 読み出し [`ArchivedMap::get`] が、インメモリの
    /// [`SpatialIdMap::get`] と同じ結果を返すことを検証する。
    ///
    /// 上下ルートの選択と枝刈りを両方踏むよう、f の正負・ヒット・ミス・
    /// 異方セル（RangeId 由来）・target がセルを覆う場合を含める。
    #[test]
    fn archived_get_matches_in_memory_get() {
        let mut map = SpatialIdMap::<Vec<u8>>::new();
        map.insert(SingleId::new(5, 3, 4, 6).unwrap(), b"upper".to_vec());
        map.insert(SingleId::new(5, -2, 4, 6).unwrap(), b"lower".to_vec());
        map.insert(
            RangeId::new(5, [1, 4], [8, 9], [5, 6]).unwrap(),
            b"range".to_vec(),
        );

        let bytes = map.to_bytes().unwrap();
        let archived = unsafe { ArchivedMap::access(&bytes) };

        let targets = [
            SingleId::new(5, 3, 4, 6).unwrap(),  // 上半分・ヒット
            SingleId::new(5, -2, 4, 6).unwrap(), // 下半分・ヒット
            SingleId::new(5, 2, 8, 5).unwrap(),  // RangeId 由来の異方セル
            SingleId::new(5, 3, 4, 7).unwrap(),  // 上半分・ミス
            SingleId::new(5, -2, 0, 0).unwrap(), // 下半分・ミス
            SingleId::new(0, 0, 0, 0).unwrap(),  // 上半分を丸ごと覆う
            SingleId::new(0, -1, 0, 0).unwrap(), // 下半分を丸ごと覆う
        ];

        for target in targets {
            for flex_target in target.clone().into_iter() {
                let mut expected: Vec<(crate::FlexId, Vec<u8>)> =
                    map.get(&target).map(|(id, v)| (id, v.clone())).collect();
                expected.sort();

                let mut actual: Vec<(crate::FlexId, Vec<u8>)> = archived
                    .get(&flex_target)
                    .into_iter()
                    .map(|(id, v)| (id, v.to_vec()))
                    .collect();
                actual.sort();

                assert_eq!(actual, expected, "target={target:?}");
            }
        }
    }
}
