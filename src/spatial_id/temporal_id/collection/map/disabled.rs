//! `temporal_id` feature 無効時の [`TemporalMap`] スタブ。
//!
//! 有効時（[`mod.rs`](super::mod) / [`impls.rs`](super::impls)）と同じ公開 API を保持しつつ、
//! 「空」か「全時間 → 1 つの値」の 2 状態だけを扱うシングルトン実装を提供する。

use alloc::vec::Vec;

use crate::{TemporalId, TemporalSet};

const DOMAIN_END: u64 = crate::Interval::WHOLE_SECONDS;

/// 時間 → 値 `V` の対応（`temporal_id` feature 無効時のスタブ）。
///
/// feature 無効時はすべての時間 ID が全時間（WHOLE）なので、マップは
/// 「空」か「全時間 → 1 つの値」の 2 状態のみをとる。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct TemporalMap<V> {
    value: Option<V>,
}

impl<V: Clone + PartialEq> TemporalMap<V> {
    /// 空。
    pub fn new() -> Self {
        Self { value: None }
    }

    /// [`TemporalId`] に値 `v` を対応させる。
    pub fn insert(&mut self, _t: &TemporalId, v: V) {
        self.value = Some(v);
    }

    /// 空かどうか。
    pub fn is_empty(&self) -> bool {
        self.value.is_none()
    }

    /// `target` の時間範囲と交差する `(TemporalId, &V)` を返す。
    ///
    /// 有効時の [`TemporalMap::get`] と同じシグネチャ。
    /// `temporal_id` feature 無効時は値があれば `(WHOLE, &v)` を 1 つ返す。
    pub fn get(&self, _target: TemporalId) -> impl Iterator<Item = (TemporalId, &V)> + '_ {
        self.value.iter().map(|v| (TemporalId::WHOLE, v))
    }

    /// その時刻が含まれるか。
    pub fn contains_unixtime(&self, _sec: u64) -> Option<&V> {
        self.value.as_ref()
    }

    /// 上書き合成（other が存在すれば other が勝つ）。
    pub fn overwrite(&self, other: &Self) -> Self {
        Self {
            value: other.value.clone().or_else(|| self.value.clone()),
        }
    }

    /// 差集合 `self - other`（時間で other を除く）。
    pub fn difference(&self, other: &Self) -> Self {
        Self {
            value: if other.value.is_some() {
                None
            } else {
                self.value.clone()
            },
        }
    }

    /// 時間集合 `set` に含まれる時間だけを残す。
    pub fn intersect_time(&self, set: &TemporalSet) -> Self {
        Self {
            value: if set.is_whole() {
                self.value.clone()
            } else {
                None
            },
        }
    }

    /// 時間集合 `set` に含まれる時間を取り除く。
    pub fn subtract_time(&self, set: &TemporalSet) -> Self {
        Self {
            value: if set.is_empty() {
                self.value.clone()
            } else {
                None
            },
        }
    }

    /// `TemporalMap` の時系列セルと値への参照のペアを走査するイテレータを返す。
    pub fn iter(&self) -> impl Iterator<Item = (TemporalId, &V)> + '_ {
        self.value.iter().map(|v| (TemporalId::WHOLE, v))
    }

    /// `TemporalMap` のすべての [`TemporalId`] を走査するイテレータを返す。
    pub fn temporal_ids(&self) -> impl Iterator<Item = TemporalId> + '_ {
        self.iter().map(|(t, _)| t)
    }

    /// `TemporalMap` のすべての値への参照を走査するイテレータを返す。
    pub fn values(&self) -> impl Iterator<Item = &V> + '_ {
        self.iter().map(|(_, v)| v)
    }

    /// 保持する [`TemporalId`] の個数を返す（O(1)）。
    pub fn len(&self) -> usize {
        self.value.iter().count()
    }

    /// 指定された時間範囲の値を取り除く。
    pub fn remove(&mut self, _t: &TemporalId) {
        self.value = None;
    }

    /// すべての時間と値の対応をクリアする。
    pub fn clear(&mut self) {
        self.value = None;
    }

    /// 正規化済みセグメント列 `(start, end, &V)` を返す（永続化・走査用の内部フック）。
    #[allow(dead_code)]
    pub(crate) fn ranges_ref(&self) -> Vec<(u64, u64, &V)> {
        self.value.iter().map(|v| (0, DOMAIN_END, v)).collect()
    }

    /// 正規化済みセグメント列から直接構築する（永続化復元用の内部フック）。
    #[cfg(feature = "persist")]
    pub(crate) fn from_raw_ranges(mut segments: Vec<(u64, u64, V)>) -> Self {
        Self {
            value: segments.pop().map(|(_, _, v)| v),
        }
    }
}

impl<V: Clone + Ord> TemporalMap<V> {
    /// 和（both は `policy` で値解決、片側はそのまま）。
    pub fn union(&self, other: &Self, policy: &crate::ConflictPolicy<V>) -> Self {
        Self {
            value: match (&self.value, &other.value) {
                (Some(a), Some(b)) => Some(policy.resolve(Some(a.clone()), b.clone())),
                (Some(a), None) => Some(a.clone()),
                (None, Some(b)) => Some(b.clone()),
                (None, None) => None,
            },
        }
    }

    /// 積集合。両方が存在する場合のみ値を保持する。
    pub fn intersection(&self, other: &Self, policy: &crate::ConflictPolicy<V>) -> Self {
        Self {
            value: if let (Some(a), Some(b)) = (&self.value, &other.value) {
                Some(policy.resolve(Some(a.clone()), b.clone()))
            } else {
                None
            },
        }
    }
}

// ── trait impl ────────────────────────────────────────────────────────────────

impl<'a, V: Clone + PartialEq + 'a> IntoIterator for &'a TemporalMap<V> {
    type Item = (TemporalId, &'a V);
    type IntoIter = core::option::IntoIter<(TemporalId, &'a V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.value
            .as_ref()
            .map(|v| (TemporalId::WHOLE, v))
            .into_iter()
    }
}

impl<V: Clone + PartialEq + 'static> IntoIterator for TemporalMap<V> {
    type Item = (TemporalId, V);
    type IntoIter = core::option::IntoIter<(TemporalId, V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.value.map(|v| (TemporalId::WHOLE, v)).into_iter()
    }
}
