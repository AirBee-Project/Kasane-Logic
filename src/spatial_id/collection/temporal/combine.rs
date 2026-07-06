//! [`Combine`]: 時間値（[`TemporalSet`] / [`TemporalMap`]）の合成規則。
//!
//! [`FlexTreeCore`](crate::FlexTreeCore) の汎用二項演算
//! （[`combine_with`](crate::FlexTreeCore::combine_with)）へ差し込むことで、
//! 空間木の走査1回で時間軸の union / intersection / difference / 上書きを行う。

use crate::spatial_id::collection::tree::node::Node;
use crate::spatial_id::collection::tree::node_ops::Combine;
use crate::spatial_id::collection::tree::ptr::SharedNode;

// ─── Combine 実装 ─────────────────────────────────────────────────────────────

/// 時間マップの上書き合成（both は時間重なりで b が勝つ、片側はそのまま）。
///
/// [`SpatialIdMap`](crate::SpatialIdMap) / [`SpatialIdTable`](crate::SpatialIdTable) の
/// 挿入（後勝ち）に使う。
pub(crate) struct TMapOverwrite;
impl<V> Combine<crate::TemporalMap<V>> for TMapOverwrite
where
    crate::TemporalMap<V>: crate::spatial_id::collection::tree::ptr::SafeValue,
    V: Clone + PartialEq,
{
    const KEEP_A_WHEN_B_EMPTY: bool = true;
    const KEEP_B_WHEN_A_EMPTY: bool = true;

    fn both(a: &crate::TemporalMap<V>, b: &crate::TemporalMap<V>) -> Option<crate::TemporalMap<V>> {
        let m = a.overwrite(b);
        if m.is_empty() { None } else { Some(m) }
    }
    fn a_only(a: &crate::TemporalMap<V>) -> Option<crate::TemporalMap<V>> {
        Some(a.clone())
    }
    fn b_only(b: &crate::TemporalMap<V>) -> Option<crate::TemporalMap<V>> {
        Some(b.clone())
    }
    fn on_identical(
        a: &SharedNode<Node<crate::TemporalMap<V>>>,
        _empty_leaf: &SharedNode<Node<crate::TemporalMap<V>>>,
    ) -> SharedNode<Node<crate::TemporalMap<V>>> {
        a.clone()
    }
}

/// 時間集合の和（both は union、片側はそのまま）。
pub(crate) struct TSetUnion;
impl Combine<crate::TemporalSet> for TSetUnion {
    const KEEP_A_WHEN_B_EMPTY: bool = true;
    const KEEP_B_WHEN_A_EMPTY: bool = true;

    fn both(a: &crate::TemporalSet, b: &crate::TemporalSet) -> Option<crate::TemporalSet> {
        Some(a.union(b))
    }
    fn a_only(a: &crate::TemporalSet) -> Option<crate::TemporalSet> {
        Some(a.clone())
    }
    fn b_only(b: &crate::TemporalSet) -> Option<crate::TemporalSet> {
        Some(b.clone())
    }
    fn on_identical(
        a: &SharedNode<Node<crate::TemporalSet>>,
        _empty_leaf: &SharedNode<Node<crate::TemporalSet>>,
    ) -> SharedNode<Node<crate::TemporalSet>> {
        a.clone()
    }
}

/// 時間集合の積（both は intersection、片側のみは不在）。
pub(crate) struct TSetIntersection;
impl Combine<crate::TemporalSet> for TSetIntersection {
    const KEEP_A_WHEN_B_EMPTY: bool = false;
    const KEEP_B_WHEN_A_EMPTY: bool = false;

    fn both(a: &crate::TemporalSet, b: &crate::TemporalSet) -> Option<crate::TemporalSet> {
        let i = a.intersection(b);
        if i.is_empty() { None } else { Some(i) }
    }
    fn a_only(_: &crate::TemporalSet) -> Option<crate::TemporalSet> {
        None
    }
    fn b_only(_: &crate::TemporalSet) -> Option<crate::TemporalSet> {
        None
    }
    fn on_identical(
        a: &SharedNode<Node<crate::TemporalSet>>,
        _empty_leaf: &SharedNode<Node<crate::TemporalSet>>,
    ) -> SharedNode<Node<crate::TemporalSet>> {
        a.clone()
    }
}

/// 時間集合の差（both は difference、a のみはそのまま、b のみは不在）。
pub(crate) struct TSetDifference;
impl Combine<crate::TemporalSet> for TSetDifference {
    const KEEP_A_WHEN_B_EMPTY: bool = true;
    const KEEP_B_WHEN_A_EMPTY: bool = false;

    fn both(a: &crate::TemporalSet, b: &crate::TemporalSet) -> Option<crate::TemporalSet> {
        let d = a.difference(b);
        if d.is_empty() { None } else { Some(d) }
    }
    fn a_only(a: &crate::TemporalSet) -> Option<crate::TemporalSet> {
        Some(a.clone())
    }
    fn b_only(_: &crate::TemporalSet) -> Option<crate::TemporalSet> {
        None
    }
    fn on_identical(
        _a: &SharedNode<Node<crate::TemporalSet>>,
        empty_leaf: &SharedNode<Node<crate::TemporalSet>>,
    ) -> SharedNode<Node<crate::TemporalSet>> {
        empty_leaf.clone()
    }
}
