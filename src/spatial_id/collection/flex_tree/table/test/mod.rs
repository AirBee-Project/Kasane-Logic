#![cfg_attr(test, allow(dead_code))]

pub mod count;
pub mod from_cells;
pub mod insert;
pub mod query;
pub mod remove;

#[cfg(test)]
use crate::{RangeId, SingleId, SpatialIdTable};

#[cfg(test)]
#[derive(Clone, Debug)]
pub(crate) enum TableEntry {
    Single(SingleId, i32),
    Range(RangeId, i32),
}

#[cfg(test)]
impl TableEntry {
    fn insert_into(&self, table: &mut SpatialIdTable<i32>) {
        match self {
            TableEntry::Single(single_id, value) => table.insert(single_id.clone(), *value),
            TableEntry::Range(range_id, value) => table.insert(range_id.clone(), *value),
        }
    }
}

#[cfg(test)]
pub(crate) fn build_table(entries: &[TableEntry]) -> SpatialIdTable<i32> {
    let mut table = SpatialIdTable::new();
    for entry in entries {
        entry.insert_into(&mut table);
    }
    table
}

#[cfg(all(test, feature = "persist"))]
mod persist_tests {
    use super::SpatialIdTable;
    use crate::{RangeId, SingleId};
    use alloc::vec::Vec;

    fn sorted(table: &SpatialIdTable<Vec<u8>>) -> Vec<(crate::FlexId, Vec<u8>)> {
        let mut v: Vec<_> = table.iter().map(|(f, val)| (f, val.clone())).collect();
        v.sort();
        v
    }

    #[test]
    fn round_trip() {
        let mut table = SpatialIdTable::<Vec<u8>>::new();
        table.insert(SingleId::new(20, 0, 0, 0).unwrap(), b"alpha".to_vec());
        table.insert(SingleId::new(20, 0, 2, 3).unwrap(), b"alpha".to_vec());
        table.insert(SingleId::new(18, 1, 5, 7).unwrap(), b"beta".to_vec());
        table.insert(
            RangeId::new(5, [1, 4], [8, 9], [5, 6]).unwrap(),
            b"gamma".to_vec(),
        );

        let bytes = table.to_bytes().unwrap();
        let restored = unsafe { SpatialIdTable::<Vec<u8>>::from_bytes(&bytes).unwrap() };

        assert_eq!(sorted(&table), sorted(&restored));
        assert_eq!(table.count(), restored.count());
    }

    #[test]
    fn round_trip_empty() {
        let table = SpatialIdTable::<Vec<u8>>::new();
        let bytes = table.to_bytes().unwrap();
        let restored = unsafe { SpatialIdTable::<Vec<u8>>::from_bytes(&bytes).unwrap() };
        assert!(restored.is_empty());
    }

    #[test]
    fn restored_is_mutable() {
        let mut table = SpatialIdTable::<Vec<u8>>::new();
        table.insert(SingleId::new(20, 0, 0, 0).unwrap(), b"alpha".to_vec());
        let bytes = table.to_bytes().unwrap();
        let mut restored = unsafe { SpatialIdTable::<Vec<u8>>::from_bytes(&bytes).unwrap() };
        let before = restored.count();
        restored.insert(SingleId::new(20, 0, 100, 100).unwrap(), b"delta".to_vec());
        assert_eq!(restored.count(), before + 1);
    }
}
