//! [`Combine`]: 時間値（[`TemporalSet`] / [`TemporalMap`]）の合成規則。
//!
//! [`FlexTree`](crate::FlexTree) の汎用二項演算
//! （[`combine_with`](crate::FlexTree::combine_with)）へ差し込むことで、
//! 空間木の走査1回で時間軸の union / intersection / difference / 上書きを行う。

use crate::spatial_id::collection::flex_tree::node::Node;
use crate::spatial_id::collection::flex_tree::node_ops::Combine;
use crate::spatial_id::collection::flex_tree::ptr::SharedNode;

// ─── Combine 実装 ─────────────────────────────────────────────────────────────

/// 時間マップの上書き合成（both は時間重なりで b が勝つ、片側はそのまま）。
///
/// [`SpatialIdMap`](crate::SpatialIdMap) / [`SpatialIdTable`](crate::SpatialIdTable) の
/// 挿入（後勝ち）に使う。
pub(crate) struct TMapOverwrite;
impl<V> Combine<crate::TemporalMap<V>> for TMapOverwrite
where
    crate::TemporalMap<V>: crate::spatial_id::collection::flex_tree::ptr::SafeValue,
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

/// 時間マップの差集合（時間で other を除く。値は self 由来）。
pub(crate) struct TMapDifference;
impl<V> Combine<crate::TemporalMap<V>> for TMapDifference
where
    crate::TemporalMap<V>: crate::spatial_id::collection::flex_tree::ptr::SafeValue,
    V: Clone + PartialEq,
{
    const KEEP_A_WHEN_B_EMPTY: bool = true;
    const KEEP_B_WHEN_A_EMPTY: bool = false;

    fn both(a: &crate::TemporalMap<V>, b: &crate::TemporalMap<V>) -> Option<crate::TemporalMap<V>> {
        let d = a.difference(b);
        if d.is_empty() { None } else { Some(d) }
    }
    fn a_only(a: &crate::TemporalMap<V>) -> Option<crate::TemporalMap<V>> {
        Some(a.clone())
    }
    fn b_only(_: &crate::TemporalMap<V>) -> Option<crate::TemporalMap<V>> {
        None
    }
    fn on_identical(
        _a: &SharedNode<Node<crate::TemporalMap<V>>>,
        empty_leaf: &SharedNode<Node<crate::TemporalMap<V>>>,
    ) -> SharedNode<Node<crate::TemporalMap<V>>> {
        empty_leaf.clone()
    }
}

/// 時間マップの積集合（V=() 専用）。
pub(crate) struct TMapIntersection;
impl Combine<crate::TemporalMap<()>> for TMapIntersection {
    const KEEP_A_WHEN_B_EMPTY: bool = false;
    const KEEP_B_WHEN_A_EMPTY: bool = false;

    fn both(
        a: &crate::TemporalMap<()>,
        b: &crate::TemporalMap<()>,
    ) -> Option<crate::TemporalMap<()>> {
        let i = a.intersection(b, &crate::ConflictPolicy::Overwrite);
        if i.is_empty() { None } else { Some(i) }
    }
    fn a_only(_: &crate::TemporalMap<()>) -> Option<crate::TemporalMap<()>> {
        None
    }
    fn b_only(_: &crate::TemporalMap<()>) -> Option<crate::TemporalMap<()>> {
        None
    }
    fn on_identical(
        a: &SharedNode<Node<crate::TemporalMap<()>>>,
        _empty_leaf: &SharedNode<Node<crate::TemporalMap<()>>>,
    ) -> SharedNode<Node<crate::TemporalMap<()>>> {
        a.clone()
    }
}
