//! [`TemporalMap`]: 時間 → 値 `V` の対応（1次元）。
//!
//! [`TemporalSet`](crate::TemporalSet) の値付き版。内部は **正規化済み**
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
            for c in TemporalId::decompose(*s, *e) {
                out.push((c, v.clone()));
            }
        }
        out
    }

    /// [`cells`](Self::cells) の参照版（値をクローンしない）。
    pub fn cells_ref(&self) -> Vec<(TemporalId, &V)> {
        let mut out = Vec::new();
        for (s, e, v) in &self.segments {
            for c in TemporalId::decompose(*s, *e) {
                out.push((c, v));
            }
        }
        out
    }

    /// `window` に限定したセル列を参照で返す（`(self ∩ window)` の分解）。
    pub fn cells_in_window_ref(&self, window: &TemporalId) -> Vec<(TemporalId, &V)> {
        let (w0, w1) = (window.start_unixtime(), window.end_unixtime_exclusive());
        let mut out = Vec::new();
        for (s, e, v) in &self.segments {
            let cs = (*s).max(w0);
            let ce = (*e).min(w1);
            if cs < ce {
                for c in TemporalId::decompose(cs, ce) {
                    out.push((c, v));
                }
            }
        }
        out
    }

    /// 正規化済みセグメント列 `(start, end, &V)` を返す（永続化・走査用の内部フック）。
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
mod tests {
    use super::TemporalMap;
    use crate::{ConflictPolicy, TemporalId};
    use alloc::collections::BTreeMap;

    /// 秒 → 値 の写像へ展開（有界ドメイン）。
    fn secmap(m: &TemporalMap<i32>) -> BTreeMap<u64, i32> {
        let mut out = BTreeMap::new();
        for &(s, e, v) in &m.segments {
            for sec in s..e {
                out.insert(sec, v);
            }
        }
        out
    }

    fn seg(i: u64, t: u64, v: i32) -> TemporalMap<i32> {
        TemporalMap::from_temporal(&TemporalId::from_seconds(i, t).unwrap(), v)
    }

    /// 正規化不変条件（昇順・互いに素・隣接同値なし）。
    fn assert_normalized(m: &TemporalMap<i32>) {
        for w in m.segments.windows(2) {
            // 昇順・非重なり
            assert!(w[0].1 <= w[1].0, "overlap/order: {:?}", m.segments);
            // 隣接同値は無い
            assert!(
                !(w[0].1 == w[1].0 && w[0].2 == w[1].2),
                "adjacent equal not merged: {:?}",
                m.segments
            );
        }
        for &(s, e, _) in &m.segments {
            assert!(s < e);
        }
    }

    /// 秒写像オラクルで union/intersection/difference を照合。
    #[test]
    fn map_algebra_oracle() {
        // A: [0,120)=1, [180,240)=2   B: [60,200)=9
        let mut a = seg(60, 0, 1); // [0,60)=1 …作り込みは union で
        a = a.union(&seg(60, 1, 1), &ConflictPolicy::Overwrite); // [0,120)=1
        a = a.union(&seg(60, 3, 2), &ConflictPolicy::Overwrite); // +[180,240)=2
        let b = {
            // B = [60,200)=9 を秒単位で作る
            let mut bb = TemporalMap::new();
            for t in 60..200u64 {
                bb = bb.union(
                    &TemporalMap::from_temporal(&TemporalId::from_seconds(1, t).unwrap(), 9),
                    &ConflictPolicy::Overwrite,
                );
            }
            bb
        };

        let (sa, sb) = (secmap(&a), secmap(&b));

        // difference: A の時間から B の時間を除く（値は A）
        let d = a.difference(&b);
        assert_normalized(&d);
        let exp_d: BTreeMap<u64, i32> = sa
            .iter()
            .filter(|(k, _)| !sb.contains_key(k))
            .map(|(&k, &v)| (k, v))
            .collect();
        assert_eq!(secmap(&d), exp_d);

        // union（Overwrite=後勝ち=B優先）
        let u = a.union(&b, &ConflictPolicy::Overwrite);
        assert_normalized(&u);
        let mut exp_u = sa.clone();
        for (&k, &v) in &sb {
            exp_u.insert(k, v); // B で上書き
        }
        assert_eq!(secmap(&u), exp_u);

        // intersection（Overwrite=B優先）
        let i = a.intersection(&b, &ConflictPolicy::Overwrite);
        assert_normalized(&i);
        let exp_i: BTreeMap<u64, i32> = sb
            .iter()
            .filter(|(k, _)| sa.contains_key(k))
            .map(|(&k, &v)| (k, v))
            .collect();
        assert_eq!(secmap(&i), exp_i);

        // union（Max）
        let um = a.union(&b, &ConflictPolicy::Max);
        let mut exp_um = sa.clone();
        for (&k, &v) in &sb {
            let e = exp_um.entry(k).or_insert(v);
            *e = (*e).max(v);
        }
        assert_eq!(secmap(&um), exp_um);
    }

    /// value_at（二分探索）の照合。
    #[test]
    fn value_at_oracle() {
        let mut m = seg(60, 0, 1);
        m = m.union(&seg(60, 3, 2), &ConflictPolicy::Overwrite); // [0,60)=1, [180,240)=2
        let s = secmap(&m);
        for probe in [0u64, 30, 59, 60, 179, 180, 239, 240, 1000] {
            assert_eq!(m.value_at(probe), s.get(&probe), "value_at({probe})");
        }
    }

    /// cells 往復で被覆と値が保たれる。
    #[test]
    fn cells_roundtrip() {
        let mut m = seg(3600, 0, 7); // [0,3600)=7
        m = m.union(&seg(60, 60, 8), &ConflictPolicy::Overwrite); // [3600,3660)=8
        let mut rebuilt = TemporalMap::new();
        for (c, v) in m.cells() {
            rebuilt = rebuilt.union(
                &TemporalMap::from_temporal(&c, v),
                &ConflictPolicy::Overwrite,
            );
        }
        assert_eq!(secmap(&m), secmap(&rebuilt));
    }
}
