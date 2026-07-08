use alloc::vec::Vec;
use core::ops::Sub;

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

    pub fn insert(&mut self, t: &TemporalId, v: V) {
        self.0
            .insert(t.start_unixtime()..t.end_unixtime_exclusive(), v);
    }

    /// 空かどうか。
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
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

    /// `TemporalMap` の時系列セルと値への参照のペアを走査するイテレータを返します。
    pub fn iter(&self) -> impl Iterator<Item = (TemporalId, &V)> + '_ {
        self.0.iter()
    }

    /// `TemporalMap` のすべての時間セル（キー）を走査するイテレータを返します。
    pub fn temporal_ids(&self) -> impl Iterator<Item = TemporalId> + '_ {
        self.iter().map(|(t, _)| t)
    }

    /// `TemporalMap` のすべての値への参照を走査するイテレータを返します。
    pub fn values(&self) -> impl Iterator<Item = &V> + '_ {
        self.iter().map(|(_, v)| v)
    }

    /// 保持する時間セルの総数を返します（O(1)）。
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// 指定秒の値への参照を取得します（二分探索）。
    pub fn get(&self, sec: u64) -> Option<&V> {
        self.0.get(sec)
    }

    /// 指定された時間範囲の値を取り除きます。
    pub fn remove(&mut self, t: &TemporalId) {
        *self = self.subtract_time(&TemporalSet::from(t));
    }

    /// すべての時間と値の対応をクリアします。
    pub fn clear(&mut self) {
        self.0 = TemporalCore::new();
    }

    /// `window` に限定したセル列を参照で返す（`(self ∩ window)` の分解）。
    pub fn temporal_ids_clipped_iter(
        &self,
        window: TemporalId,
    ) -> impl Iterator<Item = (TemporalId, &V)> + '_ {
        let (w0, w1) = (window.start_unixtime(), window.end_unixtime_exclusive());
        self.0.ranges.iter().flat_map(move |(s, e, v)| {
            let cs = (*s).max(w0);
            let ce = (*e).min(w1);
            TemporalId::from_range(cs..ce).unwrap().map(move |c| (c, v))
        })
    }

    /// 正規化済みセグメント列 `(start, end, &V)` を返す（永続化・走査用の内部フック）。
    #[cfg_attr(not(any(test, feature = "persist")), allow(dead_code))]
    pub(crate) fn segments_ref(&self) -> Vec<(u64, u64, &V)> {
        self.0.ranges_ref()
    }

    /// 正規化済みセグメント列から直接構築する（永続化復元用の内部フック）。
    ///
    /// 呼び出し側は列が正規化済み（昇順・互いに素・隣接同値マージ済み）であることを保証すること。
    #[cfg(feature = "persist")]
    pub(crate) fn from_raw_segments(segments: Vec<(u64, u64, V)>) -> Self {
        Self(TemporalCore::from_raw_ranges(segments))
    }
}

impl<V: Clone + Ord> TemporalMap<V> {
    /// 和（both は `policy` で値解決、片側はそのまま）。
    pub fn union(&self, other: &Self, policy: &ConflictPolicy<V>) -> Self {
        Self(self.0.union(&other.0, policy))
    }

    /// 積（both のみ・`policy` で値解決）。
    pub fn intersection(&self, other: &Self, policy: &ConflictPolicy<V>) -> Self {
        Self(self.0.sweep(&other.0, |a, b| match (a, b) {
            (Some(a), Some(b)) => Some(policy.resolve(Some(a.clone()), b.clone())),
            _ => None,
        }))
    }
}

impl<V: Clone + PartialEq> Sub for &TemporalMap<V> {
    type Output = TemporalMap<V>;
    fn sub(self, rhs: Self) -> Self::Output {
        self.difference(rhs)
    }
}

impl<V: Clone + PartialEq> IntoIterator for &TemporalMap<V> {
    type Item = (TemporalId, V);
    type IntoIter = alloc::vec::IntoIter<(TemporalId, V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
            .map(|(t, v)| (t, v.clone()))
            .collect::<Vec<_>>()
            .into_iter()
    }
}

#[cfg(test)]
mod tests;
