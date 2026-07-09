//! `temporal_id` feature 無効時の [`TemporalId`] 演算スタブ。
//!
//! 有効時（[`ops.rs`](super::super::ops)）と同じ公開 API を保持する。
//! feature 無効時はすべての時間 ID が全時間（WHOLE）なので、
//! 交差は常に WHOLE、差集合は常に空となる。

use crate::TemporalId;

impl TemporalId {
    /// 2 つの [`TemporalId`] の交差を計算する。
    ///
    /// `temporal_id` feature 無効時は常に `Some(WHOLE)` を返す。
    pub fn intersection(&self, _other: TemporalId) -> Option<TemporalId> {
        Some(TemporalId::WHOLE)
    }

    /// 相手の [`TemporalId`] との差集合（self − other）を計算し、イテレータとして返す。
    ///
    /// `temporal_id` feature 無効時は常に空（WHOLE − WHOLE = 空）を返す。
    pub fn difference(&self, _other: TemporalId) -> impl Iterator<Item = TemporalId> {
        core::iter::empty()
    }

    /// ある範囲に限定した差集合 `(self ∩ window) − other` を返す。
    ///
    /// `temporal_id` feature 無効時は常に空を返す。
    pub fn difference_clipped(
        &self,
        _other: &TemporalId,
        _window: &TemporalId,
    ) -> alloc::vec::Vec<TemporalId> {
        alloc::vec::Vec::new()
    }

    /// `other` の時間範囲が `self` に完全に含まれるかを判定する。
    ///
    /// `temporal_id` feature 無効時は常に `true` を返す。
    pub fn contains(&self, _other: TemporalId) -> bool {
        true
    }
}
