#[cfg(test)]
mod tests {
    use super::super::{TableEntry, build_table};
    use crate::{RangeId, SingleId};

    fn assert_len_consistent(table: &crate::SpatialIdTable<i32>) {
        assert_eq!(table.len(), table.iter().count());
    }

    /// 挿入後の len() が iter() の要素数と一致することを検証する。
    #[test]
    fn len_matches_iter_count_after_fixed_insert() {
        let table = build_table(&[
            TableEntry::Single(SingleId::new(4, 3, 2, 1).unwrap(), 10),
            TableEntry::Single(SingleId::new(4, 3, 2, 2).unwrap(), 20),
            TableEntry::Range(RangeId::new(3, [0, 1], [1, 2], [3, 3]).unwrap(), 30),
        ]);

        assert_len_consistent(&table);
    }

    /// 削除後の len() が iter() の要素数と一致することを検証する。
    #[test]
    fn len_matches_iter_count_after_remove() {
        let mut table = build_table(&[
            TableEntry::Single(SingleId::new(4, 3, 2, 1).unwrap(), 10),
            TableEntry::Single(SingleId::new(4, 3, 2, 2).unwrap(), 20),
            TableEntry::Range(RangeId::new(3, [0, 1], [1, 2], [3, 3]).unwrap(), 30),
        ]);

        let remove_target = SingleId::new(4, 3, 2, 2).unwrap();
        let _ = table.remove(&remove_target).collect::<Vec<_>>();

        assert_len_consistent(&table);
    }
}