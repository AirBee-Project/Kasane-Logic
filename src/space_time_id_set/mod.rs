use crate::space_time_id::SpaceTimeId;
use std::{collections::HashSet, hash::Hash};
pub mod single;
pub struct SpaceTimeIdSet {}
pub mod insert;
impl SpaceTimeIdSet {
    pub fn new() -> Self {
        Self {}
    }
}
