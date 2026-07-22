pub mod query;
#[cfg(test)]
mod test;

use crate::spatial_id::collection::query::execution::group_commutative::types::CommutativityInfo;
use crate::{
    Error,
    spatial_id::collection::query::traits::{UnaryOperator, WorkingTree},
};

/// 値に対する絞り込み条件。
///
/// 比較に必要なのは `Ord` だけなので、数値だけでなく文字列・真偽値でも使える。
/// 範囲は**閉区間**で、境界を `None` にするとその側は無制限になる。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValuePredicate<V> {
    /// この値を持つセルだけを残す。
    Equals(V),
    /// `min..=max` に入る値のセルだけを残す。
    InRange { min: Option<V>, max: Option<V> },
    /// `min..=max` に入る値のセルを取り除く（範囲外だけを残す）。
    NotInRange { min: Option<V>, max: Option<V> },
}

impl<V: Ord> ValuePredicate<V> {
    /// 範囲（閉区間）に入るか。
    fn in_range(value: &V, min: &Option<V>, max: &Option<V>) -> bool {
        min.as_ref().is_none_or(|m| value >= m) && max.as_ref().is_none_or(|m| value <= m)
    }

    /// この値のセルを残すか。
    pub fn matches(&self, value: &V) -> bool {
        match self {
            ValuePredicate::Equals(target) => value == target,
            ValuePredicate::InRange { min, max } => Self::in_range(value, min, max),
            ValuePredicate::NotInRange { min, max } => !Self::in_range(value, min, max),
        }
    }

    /// 範囲指定が成立しているか（`min > max` を弾く）。
    fn validate(&self) -> Result<(), Error> {
        let (min, max) = match self {
            ValuePredicate::Equals(_) => return Ok(()),
            ValuePredicate::InRange { min, max } | ValuePredicate::NotInRange { min, max } => {
                (min, max)
            }
        };
        if let (Some(min), Some(max)) = (min, max)
            && min > max
        {
            return Err(Error::InvalidQueryParameter(
                "value range lower bound is greater than upper bound",
            ));
        }
        Ok(())
    }
}

/// 値の条件に一致するセルだけを残す単項演算子。
///
/// 空間的な形は変えず、条件から外れたセルを取り除くだけ。
pub struct RetainValues<V> {
    predicate: ValuePredicate<V>,
}

impl<V> RetainValues<V> {
    pub fn new(predicate: ValuePredicate<V>) -> Self {
        Self { predicate }
    }
}

impl<W> UnaryOperator<W> for RetainValues<W::Value>
where
    W: WorkingTree,
    W::Value: Ord + 'static,
{
    fn validate(&self) -> Result<(), Error> {
        self.predicate.validate()
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }

    fn run(&self, target: &mut W) -> Result<(), Error> {
        // 条件から外れたセルは空を返す＝組み直し後の木に含まれない。
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
        // 値だけを見る演算なので、出力領域＝必要な入力領域。
        alloc::vec![bounds]
    }

    fn commutativity_info(&self) -> CommutativityInfo {
        // 値を書き換える演算子（falloff / extrude）や値を集約する zoom_out と入れ替えると
        // 結果が変わるため、並べ替えの対象にはしない。
        // 「フィルタを前倒しして読み取り量を減らす」最適化は可換グループではなく
        // ソースへの述語 pushdown として扱うのが正しい。
        CommutativityInfo::none()
    }

    fn expansion_ratio(&self) -> f32 {
        // 必ず減る（増えることはない）。
        0.5
    }

    fn fmt_op(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.predicate {
            ValuePredicate::Equals(_) => write!(f, "retain_values(== v)"),
            ValuePredicate::InRange { .. } => write!(f, "retain_values(in range)"),
            ValuePredicate::NotInRange { .. } => write!(f, "retain_values(not in range)"),
        }
    }
}
