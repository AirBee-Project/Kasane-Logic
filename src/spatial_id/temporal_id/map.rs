//! [`TemporalMap`]: 時間 → 値 `V` の対応（1次元）。
//!
//! [`TemporalSet`](crate::TemporalSet) の値付き版。内部は **正規化済み**
//! （昇順・互いに素・隣接同値マージ）の `(start, end, V)` セグメント列。
//! union / intersection / difference は境界イベント走査（sweep）で厳密に行い、
//! 重なりの値衝突は [`ConflictPolicy`] で解決する。出力（[`cells`](TemporalMap::cells)）は
//! カレンダーセル列 `(TemporalId, V)` へ最小分解する。

use alloc::vec::Vec;

use crate::{ConflictPolicy, TemporalId};

const WHOLE_END: u64 = u64::MAX;

/// 時間 → 値 `V` の対応。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
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
        let s = t.start_unixstamp();
        let e = t.end_unixtime_exclusive().min(WHOLE_END as u128) as u64;
        if s >= e {
            Self::new()
        } else {
            Self {
                segments: alloc::vec![(s, e, v)],
            }
        }
    }

    /// 空かどうか。
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// 指定秒の値。
    pub fn value_at(&self, sec: u64) -> Option<&V> {
        self.segments
            .iter()
            .find(|(s, e, _)| *s <= sec && sec < *e)
            .map(|(_, _, v)| v)
    }

    /// 境界イベント走査。各素区間 `[p, q)` について `self`/`other` の値を `combine` で合成する。
    fn sweep<F>(&self, other: &Self, combine: F) -> Self
    where
        F: Fn(Option<&V>, Option<&V>) -> Option<V>,
    {
        let mut pts: Vec<u64> =
            Vec::with_capacity((self.segments.len() + other.segments.len()) * 2);
        for (s, e, _) in &self.segments {
            pts.push(*s);
            pts.push(*e);
        }
        for (s, e, _) in &other.segments {
            pts.push(*s);
            pts.push(*e);
        }
        pts.sort_unstable();
        pts.dedup();

        let mut out: Vec<(u64, u64, V)> = Vec::new();
        for w in pts.windows(2) {
            let (p, q) = (w[0], w[1]);
            if let Some(v) = combine(self.value_at(p), other.value_at(p)) {
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

    /// 全セグメントをカレンダーセル列 `(TemporalId, V)` へ最小分解する。
    pub fn cells(&self) -> Vec<(TemporalId, V)> {
        let mut out = Vec::new();
        for (s, e, v) in &self.segments {
            if *s == 0 && *e == WHOLE_END {
                out.push((TemporalId::WHOLE, v.clone()));
            } else if let Ok(cells) = TemporalId::from_range(*s, *e) {
                for c in cells {
                    out.push((c, v.clone()));
                }
            }
        }
        out
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
        TemporalMap::from_temporal(&TemporalId::new(i, t).unwrap(), v)
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
        let b = TemporalMap::from_temporal(&TemporalId::new(1, 60).unwrap(), 9); // [60,61)=9
        let b = {
            // B = [60,200)=9 を分単位で作る
            let mut bb = TemporalMap::new();
            for t in 1..200u64 {
                if (60..200).contains(&t) {
                    bb = bb.union(
                        &TemporalMap::from_temporal(&TemporalId::new(1, t).unwrap(), 9),
                        &ConflictPolicy::Overwrite,
                    );
                }
            }
            let _ = b;
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
