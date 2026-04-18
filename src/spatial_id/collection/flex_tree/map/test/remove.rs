#[cfg(test)]
mod tests {
    use super::super::{MapEntry, build_map};
    use crate::SingleId;

    /// remove() 後に count() と iter() の整合性が保たれることを検証する。
    #[test]
    fn remove_updates_count_and_iteration() {
        let first = SingleId::new(4, 3, 2, 1).unwrap();
        let second = SingleId::new(4, 3, 2, 2).unwrap();
        let mut map = build_map(&[
            MapEntry::Single(first.clone(), 10),
            MapEntry::Single(second.clone(), 20),
        ]);

        let removed: Vec<_> = map.remove(&first).collect();
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].1, 10);
        assert_eq!(map.count(), map.iter().count());
        assert!(map.get(&first).next().is_none());
        assert_eq!(map.get(&second).map(|(_, value)| *value).collect::<Vec<_>>(), vec![20]);
    }

    /// 値が削除された後に、iter() が残った要素だけを返すことを検証する。
    #[test]
    fn remove_value_leaves_remaining_entries_only() {
        let first = SingleId::new(4, 3, 2, 1).unwrap();
        let second = SingleId::new(4, 3, 2, 2).unwrap();
        let mut map = build_map(&[
            MapEntry::Single(first.clone(), 10),
            MapEntry::Single(second.clone(), 20),
        ]);

        let _ = map.remove(&first).collect::<Vec<_>>();
        let remaining: Vec<_> = map.iter().map(|(flex_id, value)| (flex_id, *value)).collect();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].1, 20);
    }
}
