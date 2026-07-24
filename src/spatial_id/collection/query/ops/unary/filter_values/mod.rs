pub mod query;
#[cfg(test)]
mod test;

use crate::spatial_id::collection::query::execution::group_commutative::types::CommutativityInfo;
use crate::{
    Error,
    spatial_id::collection::query::traits::{UnaryOperator, WorkingTree},
};

use core::ops::Bound;

/// 値に対する絞り込み条件。
///
/// 比較に必要なのは `Ord` だけなので、数値だけでなく文字列・真偽値でも使える。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValuePredicate<V> {
    /// この値を持つセルだけを残す。
    Equals(V),
    /// 範囲に入る値のセルだけを残す。
    InRange(Bound<V>, Bound<V>),
    /// 範囲に入る値のセルを取り除く（範囲外だけを残す）。
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

    /// この値のセルを残すか。
    pub fn matches(&self, value: &V) -> bool {
        match self {
            ValuePredicate::Equals(target) => value == target,
            ValuePredicate::InRange(start, end) => Self::in_range(value, start, end),
            ValuePredicate::NotInRange(start, end) => !Self::in_range(value, start, end),
        }
    }
}

/// 値の条件に一致するセルだけを残す単項演算子。
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

impl<W> UnaryOperator<W> for FilterValues<W::Value>
where
    W: WorkingTree,
    W::Value: Ord + 'static,
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

    fn run(&self, target: &mut W) -> Result<(), Error> {
        *target = target.map_rebuild(|id, value| {
            Ok(if self.predicate.matches(value) {
                Some((id, value.clone()))
            } else {
                None
            })
        })?;
        Ok(())
    }

    fn inverse_bounds(&self, bounds: crate::RangeId) -> alloc::vec::Vec<crate::RangeId> {
        alloc::vec![bounds]
    }

    fn commutativity_info(&self) -> CommutativityInfo {
        CommutativityInfo::none()
    }

    fn expansion_ratio(&self) -> f32 {
        // 必ず減る（増えることはない）。
        0.5
    }

    fn fmt_op(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.predicate {
            ValuePredicate::Equals(_) => write!(f, "filter_values(== v)"),
            ValuePredicate::InRange(_, _) => write!(f, "filter_values(in range)"),
            ValuePredicate::NotInRange(_, _) => write!(f, "filter_values(not in range)"),
        }
    }
}
