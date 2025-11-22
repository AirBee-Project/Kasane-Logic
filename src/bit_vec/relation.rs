use crate::bit_vec::HierarchicalKey;

/// - `Ancestor`  
///     - `self` が `other` を包含する上位世代である
///
/// - `Equal`  
///     - `self` と `other` が同じ世代・範囲である
///
/// - `Descendant`  
///     - `self` が `other` の下位世代である
///
/// - `Unrelated`  
///     - `self` と `other` に世代的な包含関係がない
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HierarchicalKeyRelation {
    /// self が other を包含する上位世代
    Ancestor,

    /// self と other が同じ世代・範囲
    Equal,

    /// self が other の下位世代
    Descendant,

    /// 世代的に無関係
    Unrelated,
}

impl HierarchicalKey {
    /// self と other の世代（ancestor/descendant）関係を返す
    pub fn relation(&self, other: &Self) -> HierarchicalKeyRelation {
        let self_upper = self.upper_bound();
        let other_upper = other.upper_bound();

        // Same: 完全一致
        if self == other {
            return HierarchicalKeyRelation::Equal;
        }

        // Ancestor: self が other を包含
        if self < other && other < &self_upper {
            return HierarchicalKeyRelation::Ancestor;
        }

        // Descendant: self が other の下位
        if other < self && self < &other_upper {
            return HierarchicalKeyRelation::Descendant;
        }

        // それ以外は無関係
        HierarchicalKeyRelation::Unrelated
    }
}
