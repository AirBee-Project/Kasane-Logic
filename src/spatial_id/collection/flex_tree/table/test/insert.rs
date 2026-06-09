#[allow(unused_imports)]
use alloc::boxed::Box;
#[allow(unused_imports)]
use alloc::rc::Rc;
#[allow(unused_imports)]
use alloc::string::{String, ToString};
#[allow(unused_imports)]
use alloc::vec::Vec;

#[cfg(test)]
mod tests {
    use super::super::{TableEntry, build_table};
    use crate::{FlexId, IterSingleIds, RangeId, SingleId, SpatialIdTable};

    /// 同じターゲットへ再挿入したときに、古い値が新しい値へ置き換わることを検証する。
    #[test]
    fn insert_same_target_replaces_previous_value() {
        let target = SingleId::new(4, 3, 2, 1).unwrap();
        let mut table = SpatialIdTable::new();

        table.insert(target.clone(), 10);
        table.insert(target.clone(), 20);

        let actual_values: Vec<_> = table.get(&target).map(|(_, value)| *value).collect();
        assert_eq!(actual_values, vec![20]);
        assert!(table.value_get(&10).next().is_none());
        assert_eq!(
            table.value_get(&20).collect::<Vec<_>>(),
            vec![FlexId::from(target)]
        );
    }

    /// 同じ値を持つ複数ターゲットが同じ値インデックスに集約されることを検証する。
    #[test]
    fn insert_same_value_groups_targets_under_one_value() {
        let first = SingleId::new(4, 3, 2, 1).unwrap();
        let second = SingleId::new(4, 3, 2, 2).unwrap();

        let table = build_table(&[
            TableEntry::Single(first.clone(), 10),
            TableEntry::Single(second.clone(), 10),
        ]);

        assert_eq!(
            table.value_get(&10).collect::<Vec<_>>(),
            vec![FlexId::from(first), FlexId::from(second)]
        );
    }

    /// RangeId の挿入結果が flat_single_ids() で単一セルに展開されることを検証する。
    #[test]
    fn insert_range_is_reflected_in_flat_single_ids() {
        let range = RangeId::new(3, [0, 1], [1, 1], [2, 2]).unwrap();
        let table = build_table(&[TableEntry::Range(range.clone(), 30)]);

        let actual: Vec<_> = table
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
