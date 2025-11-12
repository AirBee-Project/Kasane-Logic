pub mod format;
pub mod remove_bottom_layer;
pub mod reverse_bottom_layer;
pub mod top_prefix;
pub mod under_prefix;
///Bit列を用いて時空間IDの各次元の階層構造を管理する
#[derive(Debug, Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]

pub struct BitVec(pub Vec<u8>);

impl BitVec {
    /// Vec<u8> から BitVec を生成
    pub fn from_vec(v: Vec<u8>) -> Self {
        BitVec(v)
    }

    /// スライスから BitVec を生成
    pub fn from_slice(s: &[u8]) -> Self {
        BitVec(s.to_vec())
    }

    /// 空の BitVec を生成
    pub fn new() -> Self {
        BitVec(Vec::new())
    }
}
