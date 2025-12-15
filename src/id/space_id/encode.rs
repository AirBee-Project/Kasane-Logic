use crate::bit_vec::BitVec;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub(crate) struct EncodeID {
    pub(crate) f: Vec<BitVec>,
    pub(crate) x: Vec<BitVec>,
    pub(crate) y: Vec<BitVec>,
}
