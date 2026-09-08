pub mod query;
#[cfg(test)]
mod test;

use alloc::boxed::Box;

use crate::{
    Error, RangeId,
    spatial_id::collection::{
        flex_tree::core::SafeValue,
        query::{UnaryOperator, ValueIter, cancellation::CancellationToken},
    },
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

    fn run<'a>(
        &'a self,
        input: ValueIter<'a, V>,
        _target: RangeId,
        token: CancellationToken,
    ) -> Result<ValueIter<'a, V>, Error> {
        let mut counter = 0u32;
        Ok(Box::new(
            input
                .map_while(move |item| token.check_amortized(&mut counter).ok().map(|_| item))
                .filter(move |(_, value)| self.predicate.matches(value)),
        ))
    }
}
