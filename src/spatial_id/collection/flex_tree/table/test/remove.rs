#[cfg(test)]
mod tests {
    use super::super::{TableEntry, build_table};
    use crate::{FlexId, SingleId};

    /// remove() 後に index と len() が整合していることを検証する。
    #[test]
    fn remove_updates_indexes_and_len() {
        let first = SingleId::new(4, 3, 2, 1).unwrap();
        let second = SingleId::new(4, 3, 2, 2).unwrap();
        let mut table = build_table(&[
            TableEntry::Single(first.clone(), 10),
            TableEntry::Single(second.clone(), 20),
        ]);

        let removed: Vec<_> = table.remove(&first).collect();
        assert_eq!(removed.len(), 1);
        let (flex_id, value) = &removed[0];
        assert_eq!(*value, 10);
        assert_eq!(flex_id.f_zoomlevel(), 4);
        assert_eq!(flex_id.f_index(), 3);
        assert_eq!(flex_id.x_zoomlevel(), 4);
        assert_eq!(flex_id.x_index(), 2);
        assert_eq!(flex_id.y_zoomlevel(), 4);
        assert_eq!(flex_id.y_index(), 1);
        assert!(table.value_get(&10).next().is_none());
        assert_eq!(
            table.value_get(&20).collect::<Vec<_>>(),
            vec![FlexId::from(second)]
        );
        assert_eq!(table.len(), table.iter().count());
    }

    /// 値を削除した後に、values() が残存値だけを返すことを検証する。
    #[test]
    fn remove_value_then_values_only_contains_remaining_values() {
        let first = SingleId::new(4, 3, 2, 1).unwrap();
        let second = SingleId::new(4, 3, 2, 2).unwrap();
        let mut table = build_table(&[
            TableEntry::Single(first.clone(), 10),
            TableEntry::Single(second.clone(), 20),
        ]);

        let _ = table.remove(&first).collect::<Vec<_>>();
        assert_eq!(table.values().copied().collect::<Vec<_>>(), vec![20]);
    }
}
