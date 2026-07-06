#![cfg_attr(test, allow(dead_code))]

pub mod count;
pub mod from_cells;
pub mod insert;
pub mod query;
pub mod remove;
pub mod temporal;

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
