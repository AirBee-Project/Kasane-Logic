use crate::{TemporalId, TemporalSet, spatial_id::temporal_id::collection::core::TemporalCore};
use alloc::vec::Vec;

pub mod impls;

#[cfg(test)]
mod tests;

/// 時間 → 値 `V` の対応。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct TemporalMap<V>(TemporalCore<V>);

impl<V: Clone + PartialEq> TemporalMap<V> {
    /// 空。
    pub fn new() -> Self {
        Self(TemporalCore::new())
    }

    pub fn insert(&mut self, t: &TemporalId, v: V) {
        self.0
            .insert(t.start_unixtime()..t.end_unixtime_exclusive(), v);
    }

    pub fn get(&self, target: TemporalId) -> impl Iterator<Item = (TemporalId, &V)> + '_ {
        let (w0, w1) = (target.start_unixtime(), target.end_unixtime_exclusive());
        self.0.ranges().iter().flat_map(move |(s, e, v)| {
            let cs = (*s).max(w0);
            let ce = (*e).min(w1);
            TemporalId::from_range(cs..ce).unwrap().map(move |c| (c, v))
        })
    }

    /// その時刻が含まれるか
    pub fn contains_unixtime(&self, sec: u64) -> Option<&V> {
        self.0.contains_unixtime_range(sec).map(|(_, _, v)| v)
    }

    /// 空かどうか。
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// 差集合 `self - other`
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

    /// 時間集合 `set` に含まれる時間だけを残す。
    pub fn intersect_time(&self, set: &TemporalSet) -> Self {
        Self(self.0.intersect_time(&set.0))
    }

    /// 時間集合 `set` に含まれる時間を取り除く。
    pub fn subtract_time(&self, set: &TemporalSet) -> Self {
        Self(self.0.subtract_time(&set.0))
    }

    /// `TemporalMap` の時系列セルと値への参照のペアを走査するイテレータを返します。
    pub fn iter(&self) -> impl Iterator<Item = (TemporalId, &V)> + '_ {
        self.0.iter()
    }

    /// `TemporalMap` のすべての[TemporalId]を走査するイテレータを返します。
    pub fn temporal_ids(&self) -> impl Iterator<Item = TemporalId> + '_ {
        self.iter().map(|(t, _)| t)
    }

    /// `TemporalMap` のすべての値への参照を走査するイテレータを返します。
    pub fn values(&self) -> impl Iterator<Item = &V> + '_ {
        self.iter().map(|(_, v)| v)
    }

    /// 保持する[TemporalId]の個数を返します（O(1)）。
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// 指定された時間範囲の値を取り除きます。
    pub fn remove(&mut self, t: &TemporalId) {
        *self = self.subtract_time(&TemporalSet::from(t));
    }

    /// すべての時間と値の対応をクリアします。
    pub fn clear(&mut self) {
        self.0 = TemporalCore::new();
    }

    /// 正規化済みセグメント列 `(start, end, &V)` を返す（永続化・走査用の内部フック）。
    #[cfg_attr(not(any(test, feature = "persist")), allow(dead_code))]
    pub(crate) fn ranges_ref(&self) -> Vec<(u64, u64, &V)> {
        self.0.ranges_ref()
    }

    /// 正規化済みセグメント列から直接構築する（永続化復元用の内部フック）。
    ///
    /// 呼び出し側は列が正規化済み（昇順・互いに素・隣接同値マージ済み）であることを保証すること。
    #[cfg(feature = "persist")]
    pub(crate) fn from_raw_ranges(segments: Vec<(u64, u64, V)>) -> Self {
        Self(TemporalCore::from_raw_ranges(segments))
    }
}
