use crate::{ConflictPolicy, TemporalId};
use alloc::vec::Vec;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
/// Coreでは[TemporalId]は扱わない。このレイヤーでは始点と終点で考える。
/// このレイヤーでは数のキャッシュなども担当する。
pub(crate) struct TemporalCore<V> {
    // Vecの中身は（開始時刻,終了時刻,値）となっている。
    // 開始時刻と終了時刻から[Interval]のサイズが自動的に特定できます。
    ranges: Vec<(u64, u64, V)>,
    cached_len: usize,
}

impl<V: Clone + PartialEq> TemporalCore<V> {
    /// 空の[TemporalCore]を作成する。
    pub(crate) fn new() -> Self {
        Self {
            ranges: Vec::new(),
            cached_len: 0,
        }
    }

    /// [TemporalCore]に新しい範囲を挿入する。
    pub(crate) fn insert(&mut self, range: core::ops::Range<u64>, v: V) {
        let s = range.start;
        let e = range.end;
        if s >= e {
            return;
        }
        let other = Self {
            ranges: alloc::vec![(s, e, v.clone())],
            cached_len: TemporalId::count_range(s..e),
        };
        *self = self.overwrite(&other);
    }

    /// 空かどうか。
    pub(crate) fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    /// 正規化済みセグメント列を借用で返す。
    pub(crate) fn ranges(&self) -> &[(u64, u64, V)] {
        &self.ranges
    }

    /// 指定秒が含まれる範囲とその値を取得します。
    pub(crate) fn contains_unixtime_range(&self, sec: u64) -> Option<(u64, u64, &V)> {
        let idx = self.ranges.partition_point(|(s, _, _)| *s <= sec);
        if idx == 0 {
            return None;
        }
        let (s, e, v) = &self.ranges[idx - 1];
        (sec < *e).then_some((*s, *e, v))
    }

    /// 境界イベント走査。各素区間 `[p, q)` について `self`/`other` の値を `combine` で合成する。
    ///
    /// 両列とも正規化済み（昇順）なので、境界点を昇順に処理しながら
    /// 各列のカーソルを単調に進める（全体 O(n + m)、ソート不要）。
    pub(crate) fn sweep<F>(&self, other: &Self, combine: F) -> Self
    where
        F: Fn(Option<&V>, Option<&V>) -> Option<V>,
    {
        let (a, b) = (&self.ranges, &other.ranges);
        let mut pts: Vec<u64> = Vec::with_capacity((a.len() + b.len()) * 2);

        let (mut ia, mut ib) = (0, 0);
        let mut add_pt = |p: u64| {
            if pts.last() != Some(&p) {
                pts.push(p);
            }
        };

        while ia < a.len() * 2 && ib < b.len() * 2 {
            let pa = if ia % 2 == 0 {
                a[ia / 2].0
            } else {
                a[ia / 2].1
            };
            let pb = if ib % 2 == 0 {
                b[ib / 2].0
            } else {
                b[ib / 2].1
            };
            if pa < pb {
                add_pt(pa);
                ia += 1;
            } else if pb < pa {
                add_pt(pb);
                ib += 1;
            } else {
                add_pt(pa);
                ia += 1;
                ib += 1;
            }
        }
        while ia < a.len() * 2 {
            let pa = if ia % 2 == 0 {
                a[ia / 2].0
            } else {
                a[ia / 2].1
            };
            add_pt(pa);
            ia += 1;
        }
        while ib < b.len() * 2 {
            let pb = if ib % 2 == 0 {
                b[ib / 2].0
            } else {
                b[ib / 2].1
            };
            add_pt(pb);
            ib += 1;
        }

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
        let cached_len = out
            .iter()
            .map(|(s, e, _)| TemporalId::count_range(*s..*e))
            .sum();
        Self {
            ranges: out,
            cached_len,
        }
    }

    /// 差集合 `self - other`（時間で other を除く。値は self 由来）。
    pub(crate) fn difference(&self, other: &Self) -> Self {
        self.sweep(other, |a, b| match (a, b) {
            (Some(a), None) => Some(a.clone()),
            _ => None,
        })
    }

    /// 上書き合成。時間が重なる部分は `other` の値が勝ち、重ならない部分は各自の値を保つ。
    pub(crate) fn overwrite(&self, other: &Self) -> Self {
        self.sweep(other, |a, b| b.or(a).cloned())
    }

    /// 時間窓 `window`（値なしの区間列）に含まれる時間だけを残す（値は self 由来）。
    pub(crate) fn intersect_time(&self, window: &TemporalCore<()>) -> Self {
        let iv = &window.ranges;
        let mut out = Vec::new();
        let mut j = 0usize;
        for (s, e, v) in &self.ranges {
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
        let cached_len = out
            .iter()
            .map(|(s, e, _)| TemporalId::count_range(*s..*e))
            .sum();
        Self {
            ranges: out,
            cached_len,
        }
    }

    /// 時間窓 `window`（値なしの区間列）に含まれる時間を取り除く（値は self 由来）。
    pub(crate) fn subtract_time(&self, window: &TemporalCore<()>) -> Self {
        let iv = &window.ranges;
        let mut out = Vec::new();
        let mut j = 0usize;
        for (s, e, v) in &self.ranges {
            let mut cur = *s;
            while j < iv.len() && iv[j].1 <= cur {
                j += 1;
            }
            let mut k = j;
            while k < iv.len() && iv[k].0 < *e {
                let (b_s, b_e) = (iv[k].0, iv[k].1);
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
        let cached_len = out
            .iter()
            .map(|(s, e, _)| TemporalId::count_range(*s..*e))
            .sum();
        Self {
            ranges: out,
            cached_len,
        }
    }

    /// 保持する時間セルの総数を返します（O(1)）。
    pub(crate) fn len(&self) -> usize {
        self.cached_len
    }

    /// [`cells_ref`](Self::cells_ref) の遅延イテレータ版（中間 `Vec` を作らない）。
    pub(crate) fn iter(&self) -> impl Iterator<Item = (TemporalId, &V)> + '_ {
        self.ranges
            .iter()
            .flat_map(|(s, e, v)| TemporalId::from_range(*s..*e).unwrap().map(move |c| (c, v)))
    }

    /// 正規化済みセグメント列 `(start, end, &V)` を返す（永続化・走査用の内部フック）。
    #[cfg_attr(not(any(test, feature = "persist")), allow(dead_code))]
    pub(crate) fn ranges_ref(&self) -> Vec<(u64, u64, &V)> {
        self.ranges.iter().map(|(s, e, v)| (*s, *e, v)).collect()
    }

    /// 正規化済みセグメント列から直接構築する（永続化復元用の内部フック）。
    ///
    /// 呼び出し側は列が正規化済み（昇順・互いに素・隣接同値マージ済み）であることを保証すること。
    #[cfg(feature = "persist")]
    pub(crate) fn from_raw_ranges(ranges: Vec<(u64, u64, V)>) -> Self {
        let cached_len = ranges
            .iter()
            .map(|(s, e, _)| TemporalId::count_range(*s..*e))
            .sum();
        Self { ranges, cached_len }
    }
}

impl<V: Clone + Ord> TemporalCore<V> {
    /// 和（both は `policy` で値解決、片側はそのまま）。
    pub(crate) fn union(&self, other: &Self, policy: &ConflictPolicy<V>) -> Self {
        self.sweep(other, |a, b| match (a, b) {
            (Some(a), Some(b)) => Some(policy.resolve(Some(a.clone()), b.clone())),
            (Some(a), None) => Some(a.clone()),
            (None, Some(b)) => Some(b.clone()),
            (None, None) => None,
        })
    }

    /// 積（both のみ・`policy` で値解決）。
    #[allow(dead_code)]
    pub(crate) fn intersection(&self, other: &Self, policy: &ConflictPolicy<V>) -> Self {
        self.sweep(other, |a, b| match (a, b) {
            (Some(a), Some(b)) => Some(policy.resolve(Some(a.clone()), b.clone())),
            _ => None,
        })
    }
}
