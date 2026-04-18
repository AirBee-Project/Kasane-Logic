#[cfg(test)]
mod tests {
    use super::super::{MapEntry, build_map};
    use crate::{RangeId, SingleId};

    fn assert_count_consistent(map: &crate::FlexTreeMap<i32>) {
        assert_eq!(map.count(), map.iter().count());
    }

    /// 挿入後の count() が iter() の要素数と一致することを検証する。
    #[test]
    fn count_matches_iter_count_after_fixed_insert() {
        let map = build_map(&[
            MapEntry::Single(SingleId::new(4, 3, 2, 1).unwrap(), 10),
            MapEntry::Single(SingleId::new(4, 3, 2, 2).unwrap(), 20),
            MapEntry::Range(RangeId::new(3, [0, 1], [1, 2], [3, 3]).unwrap(), 30),
        ]);

        assert_count_consistent(&map);
    }

    /// 削除後の count() が iter() の要素数と一致することを検証する。
    #[test]
    fn count_matches_iter_count_after_remove() {
        let mut map = build_map(&[
            MapEntry::Single(SingleId::new(4, 3, 2, 1).unwrap(), 10),
            MapEntry::Single(SingleId::new(4, 3, 2, 2).unwrap(), 20),
            MapEntry::Range(RangeId::new(3, [0, 1], [1, 2], [3, 3]).unwrap(), 30),
        ]);

        let remove_target = SingleId::new(4, 3, 2, 2).unwrap();
        let _ = map.remove(&remove_target).collect::<Vec<_>>();

        assert_count_consistent(&map);
    }
}
