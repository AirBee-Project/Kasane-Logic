use crate::bit_vec::BitVec;

/// - `Greater`  
///     - `self` が `other` より大きな範囲である
///
/// - `Equal`  
///     - `self` と `other` が表す範囲が等価である
///
/// - `Less`  
///     - `self` が `other` より小さな範囲である
///
/// - `Unrelated`  
///     - `self` と `other` に重なりがない
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitVecRelation {
    /// `self` が `other` より大きな範囲である
    Greater,

    /// `self` と `other` が表す範囲が等価である
    Equal,

    /// `self` が `other` より小さな範囲である
    Less,

    /// `self` と `other` に重なりがない
    Unrelated,
}

impl BitVec {
    /// `self` と `other` の prefix-range 関係を評価して返す。
    pub fn relation(&self, other: &Self) -> BitVecRelation {
        let self_upper = self.upper_bound();
        let other_upper = other.upper_bound();

        // Equal: 完全一致
        if self == other {
            return BitVecRelation::Equal;
        }

        // Greater: self が other を包含している
        // other ∈ [self, self_upper)
        if self < other && other < &self_upper {
            return BitVecRelation::Greater;
        }

        // Less: other が self を包含している
        // self ∈ [other, other_upper)
        if other < self && self < &other_upper {
            return BitVecRelation::Less;
        }

        // 上記以外は交差なし
        BitVecRelation::Unrelated
    }
}
