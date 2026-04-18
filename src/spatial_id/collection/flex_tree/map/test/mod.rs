#![cfg_attr(test, allow(dead_code))]

pub mod count;
pub mod insert;
pub mod query;
pub mod remove;

#[cfg(test)]
use crate::{FlexTreeMap, RangeId, SingleId};

#[cfg(test)]
#[derive(Clone, Debug)]
pub(crate) enum MapEntry {
    Single(SingleId, i32),
    Range(RangeId, i32),
}

#[cfg(test)]
impl MapEntry {
    fn insert_into(&self, map: &mut FlexTreeMap<i32>) {
        match self {
            MapEntry::Single(single_id, value) => map.insert(single_id.clone(), *value),
            MapEntry::Range(range_id, value) => map.insert(range_id.clone(), *value),
        }
    }
}

#[cfg(test)]
pub(crate) fn build_map(entries: &[MapEntry]) -> FlexTreeMap<i32> {
    let mut map = FlexTreeMap::new();
    for entry in entries {
        entry.insert_into(&mut map);
    }
    map
}
