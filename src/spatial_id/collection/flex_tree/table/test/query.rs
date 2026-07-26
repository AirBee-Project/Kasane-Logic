#[cfg(test)]
mod tests {
    use super::super::{TableEntry, build_table};
    use crate::{RangeId, SingleId};
    use alloc::vec::Vec;

    /// get() が指定した空間に一致する ID と値だけを返すことを検証する。
    #[test]
    fn get_returns_expected_pairs() {
        let first = SingleId::new(4, 3, 2, 1).unwrap();
        let second = SingleId::new(4, 3, 2, 2).unwrap();
        let table = build_table(&[
            TableEntry::Single(first.clone(), 10),
            TableEntry::Single(second.clone(), 20),
        ]);

        let actual: Vec<_> = table
            .get(&first)
            .map(|(flex_id, value)| (flex_id, *value))
            .collect();
        assert_eq!(actual.len(), 1);
        let (flex_id, value) = &actual[0];
        assert_eq!(*value, 10);
        assert_eq!(flex_id.f_zoomlevel(), 4);
        assert_eq!(flex_id.f_index(), 3);
        assert_eq!(flex_id.x_zoomlevel(), 4);
        assert_eq!(flex_id.x_index(), 2);
        assert_eq!(flex_id.y_zoomlevel(), 4);
        assert_eq!(flex_id.y_index(), 1);
    }

    /// value_range() が値の範囲条件に一致する組だけを順序付きで返すことを検証する。
    #[test]
    fn value_range_returns_expected_pairs_in_order() {
        let table = build_table(&[
            TableEntry::Single(SingleId::new(4, 3, 2, 1).unwrap(), 10),
            TableEntry::Single(SingleId::new(4, 3, 2, 2).unwrap(), 20),
            TableEntry::Range(RangeId::new(3, [0, 1], [1, 1], [2, 2]).unwrap(), 30),
        ]);

        let actual: Vec<_> = table
            .value_range(10..=20)
            .map(|(flex_id, value)| (flex_id, *value))
            .collect();

        assert_eq!(actual.len(), 2);
        assert_eq!(actual[0].1, 10);
        assert_eq!(actual[1].1, 20);
    }

    /// values() が保持している値を重複なく昇順で返すことを検証する。
    #[test]
    fn values_returns_unique_sorted_values() {
        let table = build_table(&[
            TableEntry::Single(SingleId::new(4, 3, 2, 1).unwrap(), 20),
            TableEntry::Single(SingleId::new(4, 3, 2, 2).unwrap(), 10),
            TableEntry::Range(RangeId::new(3, [0, 1], [1, 1], [2, 2]).unwrap(), 30),
            TableEntry::Single(SingleId::new(4, 3, 2, 3).unwrap(), 20),
        ]);

        assert_eq!(
            table.values().copied().collect::<Vec<_>>(),
            vec![10, 20, 30]
        );
    }

    /// iter() がテーブル中の全ての ID と値の組を返すことを検証する。
    #[test]
    fn iter_returns_all_pairs() {
        let table = build_table(&[
            TableEntry::Single(SingleId::new(4, 3, 2, 1).unwrap(), 10),
            TableEntry::Range(RangeId::new(3, [0, 1], [1, 1], [2, 2]).unwrap(), 30),
        ]);

        let actual: Vec<_> = table
            .iter()
            .map(|(flex_id, value)| (flex_id, *value))
            .collect();
        assert!(!actual.is_empty());
        assert!(actual.iter().any(|(_, value)| *value == 10));
        assert!(actual.iter().any(|(_, value)| *value == 30));
    }
}
