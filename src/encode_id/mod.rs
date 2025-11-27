use bincode::{Decode, Encode};

use crate::bit_vec::BitVec;
pub mod decode;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Encode, Decode)]
pub struct EncodeID {
    pub f: BitVec,
    pub x: BitVec,
    pub y: BitVec,
}
