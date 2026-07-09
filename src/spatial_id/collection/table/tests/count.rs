#[cfg(test)]
mod tests {
    use super::super::{TableEntry, build_table};
    use crate::{RangeId, SingleId};
    use alloc::vec::Vec;

    fn assert_count_consistent(table: &crate::SpatialIdTable<i32>) {
        assert_eq!(table.count(), table.iter().count());
    }

    /// 挿入後の count() が iter() の要素数と一致することを検証する。
    #[test]
    fn count_matches_iter_count_after_fixed_insert() {
        let table = build_table(&[
            TableEntry::Single(SingleId::new(4, 3, 2, 1).unwrap(), 10),
            TableEntry::Single(SingleId::new(4, 3, 2, 2).unwrap(), 20),
            TableEntry::Range(RangeId::new(3, [0, 1], [1, 2], [3, 3]).unwrap(), 30),
        ]);

        assert_count_consistent(&table);
    }

    /// 削除後の count() が iter() の要素数と一致することを検証する。
    #[test]
    fn count_matches_iter_count_after_remove() {
        let mut table = build_table(&[
            TableEntry::Single(SingleId::new(4, 3, 2, 1).unwrap(), 10),
            TableEntry::Single(SingleId::new(4, 3, 2, 2).unwrap(), 20),
            TableEntry::Range(RangeId::new(3, [0, 1], [1, 2], [3, 3]).unwrap(), 30),
        ]);

        let remove_target = SingleId::new(4, 3, 2, 2).unwrap();
        let _ = table.remove(&remove_target).collect::<Vec<_>>();

        assert_count_consistent(&table);
    }
}
