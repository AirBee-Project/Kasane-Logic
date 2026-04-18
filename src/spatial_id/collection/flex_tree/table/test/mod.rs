#![cfg_attr(test, allow(dead_code))]

pub mod count;
pub mod insert;
pub mod query;
pub mod remove;

#[cfg(test)]
use crate::{FlexTreeTable, RangeId, SingleId};

#[cfg(test)]
#[derive(Clone, Debug)]
pub(crate) enum TableEntry {
    Single(SingleId, i32),
    Range(RangeId, i32),
}

#[cfg(test)]
impl TableEntry {
    fn insert_into(&self, table: &mut FlexTreeTable<i32>) {
        match self {
            TableEntry::Single(single_id, value) => table.insert(single_id.clone(), *value),
            TableEntry::Range(range_id, value) => table.insert(range_id.clone(), *value),
        }
    }
}

#[cfg(test)]
pub(crate) fn build_table(entries: &[TableEntry]) -> FlexTreeTable<i32> {
    let mut table = FlexTreeTable::new();
    for entry in entries {
        entry.insert_into(&mut table);
    }
    table
}
