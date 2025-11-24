use crate::bit_vec::BitVec;

/// - `Ancestor`  
///     - `other` が `self` を包含する上位範囲（`other` is an ancestor of `self`）
///
/// - `Equal`  
///     - `self` と `other` が同じ範囲
///
/// - `Descendant`  
///     - `other` が `self` の下位範囲（`other` is a descendant of `self`）
///
/// - `Unrelated`  
///     - `self` と `other` は無関係
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitVecRelation {
    /// other が self を包含する（other is an ancestor of self）
    Ancestor,

    /// self と other が同じ範囲
    Equal,

    /// self が other を包含する（self is an ancestor of other, i.e., other is a descendant）
    Descendant,

    /// 無関係
    Unrelated,
}

impl BitVec {
    /// self と other の階層構造の関係を返す
    ///
    /// 返り値は `other` の `self` に対する関係を表す：
    /// - `Ancestor`: `other` が `self` の祖先（`other` が `self` を包含する）
    /// - `Descendant`: `other` が `self` の子孫（`self` が `other` を包含する）
    /// - `Equal`: `self` と `other` が同じ
    /// - `Unrelated`: 無関係
    pub fn relation(&self, other: &Self) -> BitVecRelation {
        let self_upper = self.upper_bound();
        let other_upper = other.upper_bound();

        if self == other {
            return BitVecRelation::Equal;
        }

        // self が other を包含する（self が親、other が子）
        if self < other && other < &self_upper {
            return BitVecRelation::Descendant;
        }

        // other が self を包含する（other が親、self が子）
        if other < self && self < &other_upper {
            return BitVecRelation::Ancestor;
        }

        // それ以外は無関係
        BitVecRelation::Unrelated
    }
}
