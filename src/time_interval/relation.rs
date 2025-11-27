use super::TimeInterval;
use crate::bit_vec::relation::BitVecRelation;

/// 時間区間の関係を表す列挙型
///
/// BitVecRelationと同様の概念だが、時間区間に特化している
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeIntervalRelation {
    /// other が self を完全に包含する（self ⊂ other）
    Ancestor,

    /// self と other が完全に一致する
    Equal,

    /// self が other を完全に包含する（other ⊂ self）
    Descendant,

    /// self と other は部分的に重なる（完全包含ではない）
    Overlap,

    /// self と other は完全に無関係（重ならない）
    Unrelated,
}

impl TimeIntervalRelation {
    /// TimeIntervalRelationをBitVecRelationに変換
    ///
    /// Overlapは既存のBitVecRelationにないため、Unrelatedとして扱う
    /// （これは区間が部分的に重なる場合、完全な包含関係がないことを意味する）
    pub fn to_bitvec_relation(&self) -> BitVecRelation {
        match self {
            TimeIntervalRelation::Ancestor => BitVecRelation::Ancestor,
            TimeIntervalRelation::Equal => BitVecRelation::Equal,
            TimeIntervalRelation::Descendant => BitVecRelation::Descendant,
            TimeIntervalRelation::Overlap => BitVecRelation::Unrelated,
            TimeIntervalRelation::Unrelated => BitVecRelation::Unrelated,
        }
    }
}

impl TimeInterval {
    /// self と other の関係を返す
    ///
    /// - `Ancestor`: other が self を完全に包含する
    /// - `Equal`: self と other が同じ
    /// - `Descendant`: self が other を完全に包含する
    /// - `Overlap`: 部分的に重なる
    /// - `Unrelated`: 完全に無関係
    pub fn relation(&self, other: &Self) -> TimeIntervalRelation {
        if self == other {
            return TimeIntervalRelation::Equal;
        }

        // 重ならない場合
        if !self.overlaps(other) {
            return TimeIntervalRelation::Unrelated;
        }

        // other が self を完全に包含する（self ⊂ other）
        if other.start <= self.start && self.end <= other.end {
            return TimeIntervalRelation::Ancestor;
        }

        // self が other を完全に包含する（other ⊂ self）
        if self.start <= other.start && other.end <= self.end {
            return TimeIntervalRelation::Descendant;
        }

        // それ以外は部分的な重なり
        TimeIntervalRelation::Overlap
    }

    /// BitVecRelationと互換性のある関係を返す
    pub fn bitvec_relation(&self, other: &Self) -> BitVecRelation {
        self.relation(other).to_bitvec_relation()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relation_equal() {
        let a = TimeInterval::new(10, 20);
        let b = TimeInterval::new(10, 20);
        assert_eq!(a.relation(&b), TimeIntervalRelation::Equal);
    }

    #[test]
    fn test_relation_ancestor() {
        let a = TimeInterval::new(15, 18);
        let b = TimeInterval::new(10, 20);
        assert_eq!(a.relation(&b), TimeIntervalRelation::Ancestor);
    }

    #[test]
    fn test_relation_descendant() {
        let a = TimeInterval::new(10, 20);
        let b = TimeInterval::new(15, 18);
        assert_eq!(a.relation(&b), TimeIntervalRelation::Descendant);
    }

    #[test]
    fn test_relation_overlap() {
        let a = TimeInterval::new(10, 20);
        let b = TimeInterval::new(15, 25);
        assert_eq!(a.relation(&b), TimeIntervalRelation::Overlap);
    }

    #[test]
    fn test_relation_unrelated() {
        let a = TimeInterval::new(10, 20);
        let b = TimeInterval::new(25, 30);
        assert_eq!(a.relation(&b), TimeIntervalRelation::Unrelated);
    }

    #[test]
    fn test_bitvec_relation_conversion() {
        let a = TimeInterval::new(10, 20);
        let b = TimeInterval::new(10, 20);
        assert_eq!(a.bitvec_relation(&b), BitVecRelation::Equal);

        let c = TimeInterval::new(15, 18);
        assert_eq!(a.bitvec_relation(&c), BitVecRelation::Descendant);
        assert_eq!(c.bitvec_relation(&a), BitVecRelation::Ancestor);
    }
}
