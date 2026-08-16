#[cfg(test)]
mod upsert_tests {
    use crate::{RangeId, SingleId, SpatialIdMap};
    use alloc::vec::Vec;

    #[test]
    fn upsert_writes_into_empty_map() {
        let mut m: SpatialIdMap<Vec<u8>> = SpatialIdMap::new();
        let a = SingleId::new(2, 0, 0, 0).unwrap();

        m.upsert(a.clone(), vec![1]);

        assert_eq!(
            m.get(&a).map(|(_, v)| v.clone()).collect::<Vec<_>>(),
            vec![vec![1]]
        );
    }

    #[test]
    fn upsert_keeps_the_existing_value() {
        let mut m: SpatialIdMap<Vec<u8>> = SpatialIdMap::new();
        let a = SingleId::new(2, 0, 0, 0).unwrap();

        m.insert(a.clone(), vec![1]);
        m.upsert(a.clone(), vec![2]);

        assert_eq!(
            m.get(&a).map(|(_, v)| v.clone()).collect::<Vec<_>>(),
            vec![vec![1]],
            "既存値が上書きされてはいけない"
        );
    }

    /// target が「既に値のある領域」と「まだ無い領域」の両方にまたがる場合、
    /// 占有済みの部分は既存値のまま、空きの部分にだけ新値が入ること。
    #[test]
    fn upsert_fills_only_the_empty_part_of_a_partial_overlap() {
        let mut m: SpatialIdMap<Vec<u8>> = SpatialIdMap::new();
        let occupied = SingleId::new(2, 0, 0, 0).unwrap();
        let empty = SingleId::new(2, 0, 1, 0).unwrap();
        let both = RangeId::new(2, 0, [0, 1], 0).unwrap();

        m.insert(occupied.clone(), vec![1]);
        m.upsert(both, vec![2]);

        assert_eq!(
            m.get(&occupied).map(|(_, v)| v.clone()).collect::<Vec<_>>(),
            vec![vec![1]],
            "既存側は保たれる"
        );
        assert_eq!(
            m.get(&empty).map(|(_, v)| v.clone()).collect::<Vec<_>>(),
            vec![vec![2]],
            "空き側には新値が入る"
        );
    }

    /// WriteCoalescer が複数の upsert submission を到着順に順次適用するのと同じ状況。
    /// 後から来た upsert は、先に来た upsert が既に埋めた ID を上書きしない。
    #[test]
    fn sequential_upserts_let_the_first_writer_win() {
        let mut m: SpatialIdMap<Vec<u8>> = SpatialIdMap::new();
        let a = SingleId::new(2, 0, 0, 0).unwrap();

        m.upsert(a.clone(), vec![1]);
        m.upsert(a.clone(), vec![2]);

        assert_eq!(
            m.get(&a).map(|(_, v)| v.clone()).collect::<Vec<_>>(),
            vec![vec![1]]
        );
    }
}
