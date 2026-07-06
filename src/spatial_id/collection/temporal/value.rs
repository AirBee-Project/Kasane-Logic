//! [`TemporalValue`]: 木の葉に格納される「時間付き値」の抽象。
//!
//! [`TemporalSet`]（値なし＝存在時間のみ）と [`TemporalMap<V>`](TemporalMap)
//! （時間 → 値）を統一的に扱うためのトレイト。[`SpatioTemporalCore`](super::SpatioTemporalCore)
//! のすべての操作はこのトレイトだけに依存する。

use alloc::vec::Vec;

use super::combine::{TMapOverwrite, TSetUnion};
use crate::spatial_id::collection::tree::node_ops::Combine;
use crate::{TemporalId, TemporalMap, TemporalSet};

// ─── TemporalValue トレイト ────────────────────────────────────────────────────

/// `FlexTreeCore` の葉に格納される「時間付き値」を抽象化するトレイト。
///
/// [`TemporalSet`] と [`TemporalMap<V>`](TemporalMap) の両方を統一的に扱うため、
/// [`SpatioTemporalCore`] のメソッドが依存するインターフェースを定義する。
#[allow(dead_code)]
pub(crate) trait TemporalValue: crate::spatial_id::collection::tree::ptr::SafeValue {
    /// ユーザー向けの「値」型。`TemporalSet` では `()`, `TemporalMap<V>` では `V`。
    type Payload: Clone;

    /// 挿入時に使う [`Combine`] 実装。
    type InsertCombine: Combine<Self>;

    /// 単一の [`TemporalId`] とペイロードからインスタンスを生成する。
    fn new_from_temporal(temporal: &TemporalId, payload: Self::Payload) -> Self;

    /// 全時間を表すインスタンスを生成する。
    fn new_whole(payload: Self::Payload) -> Self;

    /// 空（時間区間が何もない）かどうか。
    fn is_empty(&self) -> bool;

    /// 保持する時間セルをすべて `(TemporalId, Payload)` として所有権付きで返す。
    fn cells_into(self) -> Vec<(TemporalId, Self::Payload)>;

    /// 保持する時間セルをすべて `(TemporalId, &Payload)` として返す。
    fn cells_ref(&self) -> Vec<(TemporalId, &Self::Payload)>;

    /// `window` の時間範囲に限定した `(TemporalId, &Payload)` を返す。
    fn cells_in_window_ref(&self, window: &TemporalId) -> Vec<(TemporalId, &Self::Payload)>;

    /// 時間集合 `set` に含まれる時間を取り除いたインスタンスを返す。
    fn subtract_time(&self, set: &TemporalSet) -> Self;

    /// 時間集合 `set` に含まれる時間だけを残したインスタンスを返す。
    fn intersect_time(&self, set: &TemporalSet) -> Self;
}

impl TemporalValue for TemporalSet {
    type Payload = ();
    type InsertCombine = TSetUnion;

    fn new_from_temporal(temporal: &TemporalId, _payload: ()) -> Self {
        TemporalSet::from_temporal(temporal)
    }
    fn new_whole(_payload: ()) -> Self {
        TemporalSet::whole()
    }
    fn is_empty(&self) -> bool {
        TemporalSet::is_empty(self)
    }
    fn cells_into(self) -> Vec<(TemporalId, ())> {
        self.cells().into_iter().map(|t| (t, ())).collect()
    }
    fn cells_ref(&self) -> Vec<(TemporalId, &())> {
        // TemporalSet は値を持たないので、毎回 () を参照する必要がある。
        // static reference を使う代わりに、呼び出し側で () を使う。
        self.cells().into_iter().map(|t| (t, &())).collect()
    }
    fn cells_in_window_ref(&self, window: &TemporalId) -> Vec<(TemporalId, &())> {
        let window_set = TemporalSet::from_temporal(window);
        self.intersection(&window_set)
            .cells()
            .into_iter()
            .map(|t| (t, &()))
            .collect()
    }
    fn subtract_time(&self, set: &TemporalSet) -> Self {
        self.difference(set)
    }
    fn intersect_time(&self, set: &TemporalSet) -> Self {
        self.intersection(set)
    }
}

impl<V> TemporalValue for TemporalMap<V>
where
    V: Clone + PartialEq + crate::spatial_id::collection::tree::ptr::SafeValue,
{
    type Payload = V;
    type InsertCombine = TMapOverwrite;

    fn new_from_temporal(temporal: &TemporalId, payload: V) -> Self {
        TemporalMap::from_temporal(temporal, payload)
    }
    fn new_whole(payload: V) -> Self {
        TemporalMap::from_temporal(&TemporalId::WHOLE, payload)
    }
    fn is_empty(&self) -> bool {
        TemporalMap::is_empty(self)
    }
    fn cells_into(self) -> Vec<(TemporalId, V)> {
        TemporalMap::cells(&self)
    }
    fn cells_ref(&self) -> Vec<(TemporalId, &V)> {
        TemporalMap::cells_ref(self)
    }
    fn cells_in_window_ref(&self, window: &TemporalId) -> Vec<(TemporalId, &V)> {
        TemporalMap::cells_in_window_ref(self, window)
    }
    fn subtract_time(&self, set: &TemporalSet) -> Self {
        TemporalMap::subtract_time(self, set)
    }
    fn intersect_time(&self, set: &TemporalSet) -> Self {
        TemporalMap::intersect_time(self, set)
    }
}
