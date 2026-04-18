use core::fmt::{Display, Formatter};

use crate::error::Error;

#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct TemporalId;

impl TemporalId {
    pub const WHOLE: TemporalId = TemporalId;

    pub fn new(_i: u64, _t: [u64; 2]) -> Result<Self, Error> {
        Ok(Self::WHOLE)
    }

    pub fn is_whole(&self) -> bool {
        true
    }

    pub fn start_unixstamp(&self) -> u64 {
        0
    }

    pub fn end_unixstamp_inclusive(&self) -> u64 {
        u64::MAX
    }

    pub fn end_unixtime_exclusive(&self) -> u128 {
        (u64::MAX as u128) + 1
    }

    pub fn length_seconds(&self) -> u128 {
        self.end_unixtime_exclusive()
    }

    pub fn optimize_i(&mut self) {}

    pub fn intersection(&self, other: &TemporalId) -> Option<TemporalId> {
        if self.is_whole() && other.is_whole() {
            Some(TemporalId::WHOLE)
        } else {
            None
        }
    }

    pub fn difference(&self, other: &TemporalId) -> impl Iterator<Item = TemporalId> {
        let _ = other;
        std::iter::empty()
    }
}

impl Display for TemporalId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("WHOLE")
    }
}
