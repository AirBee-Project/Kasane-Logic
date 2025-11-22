pub mod ancestors;
pub mod flip_lowest_layer;
pub mod format;
pub mod relation;
pub mod remove_lowest_layer;
pub mod subtract;
pub mod upper_bound;

/// ビット列を用いて時空間IDの各次元の階層構造を管理する
///
/// 内部的にはバイト配列として保持し、階層ごとのビット操作を効率的に行う
#[derive(Debug, Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct BitVec(pub Vec<u8>);

impl BitVec {
    /// `Vec<u8>` から BitVec を生成
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
