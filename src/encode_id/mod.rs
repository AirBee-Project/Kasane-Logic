use crate::bit_vec::BitVec;
pub mod decode;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct EncodeID {
    pub f: BitVec,
    pub x: BitVec,
    pub y: BitVec,
}
