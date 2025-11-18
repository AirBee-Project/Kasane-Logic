pub mod division;
pub mod format;
pub mod remove_bottom_layer;
pub mod reverse_bottom_layer;
pub mod top_prefix;
pub mod under_prefix;
/// Bit列を用いて時空間IDの各次元の階層構造を管理する構造体
///
/// 時空間IDの各次元（F, X, Y）の範囲を効率的に表現するためのビットベクトル。
/// 階層的な範囲の包含関係を管理し、高速な範囲検索を可能にする。
#[derive(Debug, Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct BitVec(pub Vec<u8>);

impl Default for BitVec {
    fn default() -> Self {
        Self::new()
    }
}

impl BitVec {
    /// Vec<u8> から BitVec を生成する
    ///
    /// # 引数
    /// * `v` - u8のベクトル
    pub fn from_vec(v: Vec<u8>) -> Self {
        BitVec(v)
    }

    /// スライスから BitVec を生成する
    ///
    /// # 引数
    /// * `s` - u8のスライス
    pub fn from_slice(s: &[u8]) -> Self {
        BitVec(s.to_vec())
    }

    /// 空の BitVec を生成する
    pub fn new() -> Self {
        BitVec(Vec::new())
    }
}
