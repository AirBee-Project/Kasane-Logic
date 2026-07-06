//! [`TemporalMap`]: 時間 → 値 `V` の対応（1次元）。
//!
//! [`TemporalSet`] の値付き版。内部は **正規化済み**
//! （昇順・互いに素・隣接同値マージ）の `(start, end, V)` セグメント列。
//! union / intersection / difference は境界イベント走査（sweep）で厳密に行い、
//! 重なりの値衝突は [`ConflictPolicy`] で解決する。出力（[`cells`](TemporalMap::cells)）は
//! カレンダーセル列 `(TemporalId, V)` へ最小分解する。

use alloc::vec::Vec;

use crate::{ConflictPolicy, TemporalId, TemporalSet};

/// 時間 → 値 `V` の対応。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct TemporalMap<V> {
    /// 正規化済み（昇順・互いに素・隣接同値マージ）の `(start, end, V)`。
    segments: Vec<(u64, u64, V)>,
}

impl<V: Clone + PartialEq> TemporalMap<V> {
    /// 空。
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    /// 1つの [`TemporalId`] に値 `v` を対応させる。
    pub fn from_temporal(t: &TemporalId, v: V) -> Self {
        Self {
            segments: alloc::vec![(t.start_unixtime(), t.end_unixtime_exclusive(), v)],
        }
    }

    /// 空かどうか。
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// 指定秒の値（二分探索）。
    pub fn value_at(&self, sec: u64) -> Option<&V> {
        let idx = self.segments.partition_point(|(s, _, _)| *s <= sec);
        if idx == 0 {
            return None;
        }
        let (_, e, v) = &self.segments[idx - 1];
        (sec < *e).then_some(v)
    }

    /// 境界イベント走査。各素区間 `[p, q)` について `self`/`other` の値を `combine` で合成する。
    ///
    /// 両列とも正規化済み（昇順）なので、境界点を昇順に処理しながら
    /// 各列のカーソルを単調に進める（全体 O(n + m)、ソートを除く）。
    fn sweep<F>(&self, other: &Self, combine: F) -> Self
    where
        F: Fn(Option<&V>, Option<&V>) -> Option<V>,
    {
        let (a, b) = (&self.segments, &other.segments);
        let mut pts: Vec<u64> = Vec::with_capacity((a.len() + b.len()) * 2);
        for (s, e, _) in a {
            pts.push(*s);
            pts.push(*e);
        }
        for (s, e, _) in b {
            pts.push(*s);
            pts.push(*e);
        }
        pts.sort_unstable();
        pts.dedup();

        let (mut ia, mut ib) = (0usize, 0usize);
        let mut out: Vec<(u64, u64, V)> = Vec::new();
        for w in pts.windows(2) {
            let (p, q) = (w[0], w[1]);
            while ia < a.len() && a[ia].1 <= p {
                ia += 1;
            }
            while ib < b.len() && b[ib].1 <= p {
                ib += 1;
            }
            let av = (ia < a.len() && a[ia].0 <= p).then(|| &a[ia].2);
            let bv = (ib < b.len() && b[ib].0 <= p).then(|| &b[ib].2);
            if let Some(v) = combine(av, bv) {
                // 隣接同値なら伸ばす（正規化）。
                if let Some(last) = out.last_mut()
                    && last.1 == p
                    && last.2 == v
                {
                    last.1 = q;
                    continue;
                }
                out.push((p, q, v));
            }
        }
        Self { segments: out }
    }

    /// 差集合 `self - other`（時間で other を除く。値は self 由来）。
    pub fn difference(&self, other: &Self) -> Self {
        self.sweep(other, |a, b| match (a, b) {
            (Some(a), None) => Some(a.clone()),
            _ => None,
        })
    }

    /// 上書き合成。時間が重なる部分は `other` の値が勝ち、重ならない部分は各自の値を保つ。
    ///
    /// コレクションの挿入（後勝ち）に使う。[`union`](Self::union) の
    /// `ConflictPolicy::Overwrite` 特化版で、`V: Ord` を要求しない。
    pub fn overwrite(&self, other: &Self) -> Self {
        self.sweep(other, |a, b| b.or(a).cloned())
    }

    /// 時間集合 `set` に含まれる時間だけを残す（値は self 由来）。
    pub fn intersect_time(&self, set: &TemporalSet) -> Self {
        let iv = set.intervals();
        let mut out = Vec::new();
        let mut j = 0usize;
        for (s, e, v) in &self.segments {
            while j < iv.len() && iv[j].1 <= *s {
                j += 1;
            }
            let mut k = j;
            while k < iv.len() && iv[k].0 < *e {
                let cs = (*s).max(iv[k].0);
                let ce = (*e).min(iv[k].1);
                if cs < ce {
                    out.push((cs, ce, v.clone()));
                }
                if iv[k].1 >= *e {
                    break;
                }
                k += 1;
            }
        }
        Self { segments: out }
    }

    /// 時間集合 `set` に含まれる時間を取り除く（値は self 由来）。
    pub fn subtract_time(&self, set: &TemporalSet) -> Self {
        let iv = set.intervals();
        let mut out = Vec::new();
        let mut j = 0usize;
        for (s, e, v) in &self.segments {
            let mut cur = *s;
            while j < iv.len() && iv[j].1 <= cur {
                j += 1;
            }
            let mut k = j;
            while k < iv.len() && iv[k].0 < *e {
                let (b_s, b_e) = iv[k];
                if b_s > cur {
                    out.push((cur, b_s, v.clone()));
                }
                if b_e > cur {
                    cur = b_e;
                }
                if cur >= *e {
                    break;
                }
                k += 1;
            }
            if cur < *e {
                out.push((cur, *e, v.clone()));
            }
        }
        Self { segments: out }
    }

    /// 全セグメントを約数鎖セル列 `(TemporalId, V)` へ最小分解する。
    pub fn cells(&self) -> Vec<(TemporalId, V)> {
        let mut out = Vec::new();
        for (s, e, v) in &self.segments {
            for c in TemporalId::from_range(*s, *e).unwrap() {
                out.push((c, v.clone()));
            }
        }
        out
    }

    /// [`cells`](Self::cells) の参照版（値をクローンしない）。
    pub fn cells_ref(&self) -> Vec<(TemporalId, &V)> {
        let mut out = Vec::new();
        for (s, e, v) in &self.segments {
            for c in TemporalId::from_range(*s, *e).unwrap() {
                out.push((c, v));
            }
        }
        out
    }

    /// `window` に限定したセル列を参照で返す（`(self ∩ window)` の分解）。
    pub fn cells_clipped_ref(&self, window: &TemporalId) -> Vec<(TemporalId, &V)> {
        let (w0, w1) = (window.start_unixtime(), window.end_unixtime_exclusive());
        let mut out = Vec::new();
        for (s, e, v) in &self.segments {
            let cs = (*s).max(w0);
            let ce = (*e).min(w1);
            if cs < ce {
                for c in TemporalId::from_range(cs, ce).unwrap() {
                    out.push((c, v));
                }
            }
        }
        out
    }

    /// 正規化済みセグメント列 `(start, end, &V)` を返す（永続化・走査用の内部フック）。
    #[cfg_attr(not(any(test, feature = "persist")), allow(dead_code))]
    pub(crate) fn segments_ref(&self) -> Vec<(u64, u64, &V)> {
        self.segments.iter().map(|(s, e, v)| (*s, *e, v)).collect()
    }

    /// 正規化済みセグメント列から直接構築する（永続化復元用の内部フック）。
    ///
    /// 呼び出し側は列が正規化済み（昇順・互いに素・隣接同値マージ済み）であることを保証すること。
    #[cfg(feature = "persist")]
    pub(crate) fn from_raw_segments(segments: Vec<(u64, u64, V)>) -> Self {
        Self { segments }
    }
}

impl<V: Clone + Ord> TemporalMap<V> {
    /// 和（both は `policy` で値解決、片側はそのまま）。
    pub fn union(&self, other: &Self, policy: &ConflictPolicy<V>) -> Self {
        self.sweep(other, |a, b| match (a, b) {
            (Some(a), Some(b)) => Some(policy.resolve(Some(a.clone()), b.clone())),
            (Some(a), None) => Some(a.clone()),
            (None, Some(b)) => Some(b.clone()),
            (None, None) => None,
        })
    }

    /// 積（both のみ・`policy` で値解決）。
    pub fn intersection(&self, other: &Self, policy: &ConflictPolicy<V>) -> Self {
        self.sweep(other, |a, b| match (a, b) {
            (Some(a), Some(b)) => Some(policy.resolve(Some(a.clone()), b.clone())),
            _ => None,
        })
    }
}

#[cfg(test)]
mod tests;
