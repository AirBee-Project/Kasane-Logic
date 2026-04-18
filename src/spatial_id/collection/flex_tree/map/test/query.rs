#[cfg(test)]
mod tests {
    use super::super::{MapEntry, build_map};
    use crate::{IterSingleIds, RangeId, SingleId};

    /// get() が指定した空間に一致する ID と値だけを返すことを検証する。
    #[test]
    fn get_returns_expected_pairs() {
        let first = SingleId::new(4, 3, 2, 1).unwrap();
        let second = SingleId::new(4, 3, 2, 2).unwrap();
        let map = build_map(&[
            MapEntry::Single(first.clone(), 10),
            MapEntry::Single(second.clone(), 20),
        ]);

        let actual: Vec<_> = map
            .get(&first)
            .map(|(flex_id, value)| (flex_id, *value))
            .collect();
        assert_eq!(actual.len(), 1);
        assert_eq!(actual[0].1, 10);
        assert_eq!(actual[0].0.f_zoomlevel(), 4);
        assert_eq!(actual[0].0.f_index(), 3);
        assert_eq!(actual[0].0.x_zoomlevel(), 4);
        assert_eq!(actual[0].0.x_index(), 2);
        assert_eq!(actual[0].0.y_zoomlevel(), 4);
        assert_eq!(actual[0].0.y_index(), 1);
    }

    /// iter() がテーブル中の全ての ID と値の組を返すことを検証する。
    #[test]
    fn iter_returns_all_pairs() {
        let map = build_map(&[
            MapEntry::Single(SingleId::new(4, 3, 2, 1).unwrap(), 10),
            MapEntry::Range(RangeId::new(3, [0, 1], [1, 1], [2, 2]).unwrap(), 30),
        ]);

        let actual: Vec<_> = map
            .iter()
            .map(|(flex_id, value)| (flex_id, *value))
            .collect();
        assert!(!actual.is_empty());
        assert!(actual.iter().any(|(_, value)| *value == 10));
        assert!(actual.iter().any(|(_, value)| *value == 30));
    }

    /// flat_single_ids() が保持している領域を単一セルへ展開して返すことを検証する。
    #[test]
    fn flat_single_ids_returns_expanded_pairs() {
        let range = RangeId::new(3, [0, 1], [1, 1], [2, 2]).unwrap();
        let map = build_map(&[MapEntry::Range(range.clone(), 30)]);

        let actual: Vec<_> = map
            .flat_single_ids()
            .map(|(single_id, value)| (single_id, *value))
            .collect();
        assert_eq!(actual.len(), range.iter_single_ids().count());
        assert!(actual.iter().all(|(_, value)| *value == 30));
    }
}
