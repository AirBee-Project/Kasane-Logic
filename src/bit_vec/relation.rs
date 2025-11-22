use crate::bit_vec::BitVec;

/// - `Ancestor`  
///     - `self` が `other` を包含する上位範囲
///
/// - `Equal`  
///     - `self` と `other` が同じ範囲
///
/// - `Descendant`  
///     - `self` が `other` の下位範囲
///
/// - `Unrelated`  
///     - `self` と `other` は無関係
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitVecRelation {
    /// self が other を包含する上位範囲
    Ancestor,

    /// self と other が同じ範囲
    Equal,

    /// self が other の下位範囲
    Descendant,

    /// 無関係
    Unrelated,
}

impl BitVec {
    /// self と other の階層構造の関係を返す
    pub fn relation(&self, other: &Self) -> BitVecRelation {
        let self_upper = self.upper_bound();
        let other_upper = other.upper_bound();

        if self == other {
            return BitVecRelation::Equal;
        }

        if self < other && other < &self_upper {
            return BitVecRelation::Ancestor;
        }

        if other < self && self < &other_upper {
            return BitVecRelation::Descendant;
        }

        // それ以外は無関係
        BitVecRelation::Unrelated
    }
}
