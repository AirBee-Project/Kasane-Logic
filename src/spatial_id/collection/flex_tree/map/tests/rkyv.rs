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

    /// `ArchivedMap::get_range` が、範囲と交差する全セルを返すこと。
    ///
    /// 期待値は「全セルを走査して交差判定で絞ったもの」を真値として突き合わせる。
    #[test]
    fn archived_get_range_returns_all_intersecting_cells() {
        use crate::spatial_id::collection::query::execution::intersects_flex_range;

        let mut map: SpatialIdMap<Vec<u8>> = SpatialIdMap::new();
        for x in 0..8u32 {
            map.insert(
                SingleId::new(20, 0, 710000 + x, 500000).unwrap(),
                alloc::vec![x as u8],
            );
        }

        let target = RangeId::new(20, [0, 0], [710000, 710007], [500000, 500000]).unwrap();

        let mut expected: Vec<(crate::FlexId, Vec<u8>)> = map
            .iter()
            .filter(|(id, _)| intersects_flex_range(id, &target))
            .map(|(id, v)| (id, v.clone()))
            .collect();
        expected.sort_by(|a, b| a.0.cmp(&b.0));

        let bytes = map.to_bytes().unwrap();
        let arch = unsafe { ArchivedMap::access(&bytes) };
        let mut got: Vec<(crate::FlexId, Vec<u8>)> = arch
            .get_range(&target)
            .into_iter()
            .map(|(id, v)| (id, v.to_vec()))
            .collect();
        got.sort_by(|a, b| a.0.cmp(&b.0));

        assert_eq!(expected.len(), 8, "前提: 8セルすべてが範囲と交差する");
        assert_eq!(got, expected, "ゼロコピー側の範囲走査が一致しない");
    }

    /// 範囲の端が境界に揃っていないケースでも取りこぼさないこと。
    #[test]
    fn archived_get_range_various_windows() {
        use crate::spatial_id::collection::query::execution::intersects_flex_range;

        for (base, count) in [(790000u32, 4u32), (710000, 8), (931000, 5), (1, 3)] {
            let mut map: SpatialIdMap<Vec<u8>> = SpatialIdMap::new();
            for i in 0..count {
                map.insert(
                    SingleId::new(20, 0, base + i, 500000).unwrap(),
                    alloc::vec![i as u8],
                );
            }
            let target =
                RangeId::new(20, [0, 0], [base, base + count - 1], [500000, 500000]).unwrap();

            let mut expected: Vec<crate::FlexId> = map
                .iter()
                .filter(|(id, _)| intersects_flex_range(id, &target))
                .map(|(id, _)| id)
                .collect();
            expected.sort();

            let bytes = map.to_bytes().unwrap();
            let arch = unsafe { ArchivedMap::access(&bytes) };
            let mut got: Vec<crate::FlexId> = arch
                .get_range(&target)
                .into_iter()
                .map(|(id, _)| id)
                .collect();
            got.sort();

            assert_eq!(got, expected, "base={base} count={count} で不一致");
        }
    }
}
