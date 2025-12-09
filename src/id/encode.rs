use crate::{
    bit_vec::BitVec,
    id::space_id::{range::RangeID, single::SingleID},
};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct EncodeID {
    pub(crate) f: Vec<BitVec>,
    pub(crate) x: Vec<BitVec>,
    pub(crate) y: Vec<BitVec>,
}

impl EncodeID {
    pub fn to_range_id(&self) -> RangeID {
        todo!()
    }

    pub fn to_single_id(&self) {
        todo!()
    }
}
