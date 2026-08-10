use crate::spatial_id::collection::query::working::WorkingTree;
pub mod query;
#[cfg(test)]
mod test;

use crate::spatial_id::collection::query::execution::group_commutative::types::CommutativityInfo;
use crate::{
    Error,
    spatial_id::collection::{flex_tree::core::SafeValue, query::traits::UnaryOperator},
};

use core::ops::Bound;

/// 値に対する絞り込み条件。
///
/// 比較に必要なのは `Ord` だけなので、数値だけでなく文字列・真偽値でも使える。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValuePredicate<V> {
    /// この値を持つSegmentだけを残す。
    Equals(V),
    /// 範囲に入る値のSegmentだけを残す。
    InRange(Bound<V>, Bound<V>),
    /// 範囲に入る値のSegmentを取り除く（範囲外だけを残す）。
    NotInRange(Bound<V>, Bound<V>),
}

impl<V: Ord> ValuePredicate<V> {
    /// 範囲に入るか。
    fn in_range(value: &V, start: &Bound<V>, end: &Bound<V>) -> bool {
        let after_start = match start {
            Bound::Included(s) => value >= s,
            Bound::Excluded(s) => value > s,
            Bound::Unbounded => true,
        };
        let before_end = match end {
            Bound::Included(e) => value <= e,
            Bound::Excluded(e) => value < e,
            Bound::Unbounded => true,
        };
        after_start && before_end
    }

    /// この値のSegmentを残すか。
    pub fn matches(&self, value: &V) -> bool {
        match self {
            ValuePredicate::Equals(target) => value == target,
            ValuePredicate::InRange(start, end) => Self::in_range(value, start, end),
            ValuePredicate::NotInRange(start, end) => !Self::in_range(value, start, end),
        }
    }
}

/// 値の条件に一致するSegmentだけを残す単項演算子。
///
/// 空間的な形は変えず、条件から外れた空間IDを取り除くだけ。
pub struct FilterValues<V> {
    predicate: ValuePredicate<V>,
}

impl<V> FilterValues<V> {
    pub fn new(predicate: ValuePredicate<V>) -> Self {
        Self { predicate }
    }
}

impl<V> UnaryOperator<V> for FilterValues<V>
where
    V: SafeValue + Ord + 'static,
{
    fn validate(&self) -> Result<(), Error> {
        let (start, end) = match &self.predicate {
            ValuePredicate::Equals(_) => return Ok(()),
            ValuePredicate::InRange(s, e) | ValuePredicate::NotInRange(s, e) => (s, e),
        };
        let s_val = match start {
            Bound::Included(v) | Bound::Excluded(v) => Some(v),
            Bound::Unbounded => None,
        };
        let e_val = match end {
            Bound::Included(v) | Bound::Excluded(v) => Some(v),
            Bound::Unbounded => None,
        };
        if let (Some(s), Some(e)) = (s_val, e_val)
            && s > e
        {
            return Err(Error::InvalidQueryParameter(
                "value range lower bound is greater than upper bound",
            ));
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }

    fn run(&self, target: &mut WorkingTree<V>) -> Result<(), Error> {
        // この演算子はSegmentを取り除くだけで空間的な形を変えないので、木を平坦化して
        // 組み直す必要はない。`retain_values` は変化した経路だけを copy-on-write で
        // 作り直すため、条件を満たす部分木は `Arc` ごと保たれる。
        target
            .core_mut()
            .retain_values(|value| self.predicate.matches(value));
        Ok(())
    }

    fn inverse_bounds(&self, bounds: crate::RangeId) -> alloc::vec::Vec<crate::RangeId> {
        alloc::vec![bounds]
    }

    fn commutativity_info(&self) -> CommutativityInfo {
        CommutativityInfo::none()
    }

    fn expansion_ratio(&self) -> f64 {
        1.0 // フィルタリングでは増えないが、削除分を予測するのは難しいので 1.0
    }

    fn fmt_op(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.predicate {
            ValuePredicate::Equals(_) => write!(f, "filter_values(== v)"),
            ValuePredicate::InRange(_, _) => write!(f, "filter_values(in range)"),
            ValuePredicate::NotInRange(_, _) => write!(f, "filter_values(not in range)"),
        }
    }
}
