//! [`TemporalMap`]: 時間 → 値 `V` の対応（1次元）。
//!
//! [`TemporalSet`] の値付き版。中立エンジン [`TemporalCore<V>`] を被せた薄い newtype で、
//! 正規化済み（昇順・互いに素・隣接同値マージ）の `(start, end, V)` セグメント列を保持する。
//! union / intersection / difference は境界イベント走査（sweep）で厳密に行い、
//! 重なりの値衝突は [`ConflictPolicy`] で解決する。出力（[`cells`](TemporalMap::cells)）は
//! カレンダーセル列 `(TemporalId, V)` へ最小分解する。

use alloc::vec::Vec;

use super::temporal_core::TemporalCore;
use crate::{ConflictPolicy, TemporalId, TemporalSet};

/// 時間 → 値 `V` の対応。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct TemporalMap<V>(pub(crate) TemporalCore<V>);

impl<V: Clone + PartialEq> TemporalMap<V> {
    /// 空。
    pub fn new() -> Self {
        Self(TemporalCore::new())
    }

    /// 1つの [`TemporalId`] に値 `v` を対応させる。
    pub fn from_temporal(t: &TemporalId, v: V) -> Self {
        Self(TemporalCore::from_temporal(t, v))
    }

    /// 空かどうか。
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// 指定秒の値（二分探索）。
    pub fn value_at(&self, sec: u64) -> Option<&V> {
        self.0.value_at(sec)
    }

    /// 差集合 `self - other`（時間で other を除く。値は self 由来）。
    pub fn difference(&self, other: &Self) -> Self {
        Self(self.0.difference(&other.0))
    }

    /// 上書き合成。時間が重なる部分は `other` の値が勝ち、重ならない部分は各自の値を保つ。
    ///
    /// コレクションの挿入（後勝ち）に使う。[`union`](Self::union) の
    /// `ConflictPolicy::Overwrite` 特化版で、`V: Ord` を要求しない。
    pub fn overwrite(&self, other: &Self) -> Self {
        Self(self.0.overwrite(&other.0))
    }

    /// 時間集合 `set` に含まれる時間だけを残す（値は self 由来）。
    pub fn intersect_time(&self, set: &TemporalSet) -> Self {
        Self(self.0.intersect_time(&set.0))
    }

    /// 時間集合 `set` に含まれる時間を取り除く（値は self 由来）。
    pub fn subtract_time(&self, set: &TemporalSet) -> Self {
        Self(self.0.subtract_time(&set.0))
    }

    /// 保持する時間セルの総数を、セルを生成せずに数える（`cells().len()` と一致）。
    pub fn count_cells(&self) -> usize {
        self.0.count_cells()
    }

    /// 全セグメントを約数鎖セル列 `(TemporalId, V)` へ最小分解する。
    pub fn cells(&self) -> Vec<(TemporalId, V)> {
        self.0.cells()
    }

    /// [`cells`](Self::cells) の参照版（値をクローンしない）。
    pub fn cells_ref(&self) -> Vec<(TemporalId, &V)> {
        self.0.cells_ref()
    }

    /// [`cells_ref`](Self::cells_ref) の遅延イテレータ版（中間 `Vec` を作らない）。
    pub fn cells_ref_iter(&self) -> impl Iterator<Item = (TemporalId, &V)> + '_ {
        self.0.cells_ref_iter()
    }

    /// `window` に限定したセル列を参照で返す（`(self ∩ window)` の分解）。
    pub fn cells_clipped_ref(&self, window: &TemporalId) -> Vec<(TemporalId, &V)> {
        self.0.cells_clipped_ref(window)
    }

    /// [`cells_clipped_ref`](Self::cells_clipped_ref) の遅延イテレータ版（窓は値渡し）。
    pub fn cells_clipped_ref_iter(
        &self,
        window: TemporalId,
    ) -> impl Iterator<Item = (TemporalId, &V)> + '_ {
        self.0.cells_clipped_ref_iter(window)
    }

    /// 正規化済みセグメント列 `(start, end, &V)` を返す（永続化・走査用の内部フック）。
    #[cfg_attr(not(any(test, feature = "persist")), allow(dead_code))]
    pub(crate) fn segments_ref(&self) -> Vec<(u64, u64, &V)> {
        self.0.segments_ref()
    }

    /// 正規化済みセグメント列から直接構築する（永続化復元用の内部フック）。
    ///
    /// 呼び出し側は列が正規化済み（昇順・互いに素・隣接同値マージ済み）であることを保証すること。
    #[cfg(feature = "persist")]
    pub(crate) fn from_raw_segments(segments: Vec<(u64, u64, V)>) -> Self {
        Self(TemporalCore::from_raw_segments(segments))
    }
}

impl<V: Clone + Ord> TemporalMap<V> {
    /// 和（both は `policy` で値解決、片側はそのまま）。
    pub fn union(&self, other: &Self, policy: &ConflictPolicy<V>) -> Self {
        Self(self.0.union(&other.0, policy))
    }

    /// 積（both のみ・`policy` で値解決）。
    pub fn intersection(&self, other: &Self, policy: &ConflictPolicy<V>) -> Self {
        Self(self.0.intersection(&other.0, policy))
    }
}

#[cfg(test)]
mod tests;
