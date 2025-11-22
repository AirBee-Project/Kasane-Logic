pub mod ancestors;
pub mod flip_leaf;
pub mod format;
pub mod pop_leaf;
pub mod relation;
pub mod subtract;
pub mod upper_bound;

/// ビット列を用いて時空間IDの各次元の階層構造を管理する
///
/// 内部的にはバイト配列として保持し、階層ごとのビット操作を効率的に行う
#[derive(Debug, Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct HierarchicalKey(pub Vec<u8>);

impl HierarchicalKey {
    /// `Vec<u8>` から HierarchicalKey を生成
    pub fn from_vec(v: Vec<u8>) -> Self {
        HierarchicalKey(v)
    }

    /// スライスから HierarchicalKey を生成
    pub fn from_slice(s: &[u8]) -> Self {
        HierarchicalKey(s.to_vec())
    }

    /// 空の HierarchicalKey を生成
    pub fn new() -> Self {
        HierarchicalKey(Vec::new())
    }
}
