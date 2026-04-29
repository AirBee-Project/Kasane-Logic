#[cfg(test)]
mod tests {
    use super::super::{MapEntry, build_map};
    use crate::{IterSingleIds, RangeId, SingleId, SpatialIdMap};

    /// 同じターゲットへ再挿入したときに、古い値が新しい値へ置き換わることを検証する。
    #[test]
    fn insert_same_target_replaces_previous_value() {
        let target = SingleId::new(4, 3, 2, 1).unwrap();
        let mut map = SpatialIdMap::new();

        map.insert(target.clone(), 10);
        map.insert(target.clone(), 20);

        let actual: Vec<_> = map.get(&target).map(|(_, value)| *value).collect();
        assert_eq!(actual, vec![20]);
    }

    /// 同じ値を持つ複数ターゲットがそれぞれ保持されることを検証する。
    #[test]
    fn insert_same_value_keeps_multiple_targets() {
        let first = SingleId::new(4, 3, 2, 1).unwrap();
        let second = SingleId::new(4, 3, 2, 2).unwrap();

        let map = build_map(&[
            MapEntry::Single(first.clone(), 10),
            MapEntry::Single(second.clone(), 10),
        ]);

        let actual: Vec<_> = map
            .iter()
            .map(|(flex_id, value)| (flex_id, *value))
            .collect();
        assert_eq!(actual.len(), 2);
        assert!(actual.iter().all(|(_, value)| *value == 10));
    }

    /// RangeId の挿入結果が flat_single_ids() で単一セルに展開されることを検証する。
    #[test]
    fn insert_range_is_reflected_in_flat_single_ids() {
        let range = RangeId::new(3, [0, 1], [1, 1], [2, 2]).unwrap();
        let map = build_map(&[MapEntry::Range(range.clone(), 30)]);

        let actual: Vec<_> = map
            .flat_single_ids()
            .map(|(single_id, value)| (single_id, *value))
            .collect();
        let expected: Vec<_> = range
            .iter_single_ids()
            .map(|single_id| (single_id, 30))
            .collect();

        assert_eq!(actual, expected);
    }
}
