///Bit列を用いて時空間IDの各次元の階層構造を管理する
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct BitVec(pub(crate) Vec<u8>);

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
