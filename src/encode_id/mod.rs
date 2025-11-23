use crate::bit_vec::BitVec;
pub mod decode;
pub struct EncodeID {
    pub f: BitVec,
    pub x: BitVec,
    pub y: BitVec,
}
