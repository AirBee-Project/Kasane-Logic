//! [`TemporalCore`]: 時間区間代数の中立エンジン。
//!
//! 正規化済み（昇順・互いに素・隣接同値マージ）の `(start, end, V)` セグメント列を
//! 保持し、境界イベント走査（sweep）で union / intersection / difference / overwrite を
//! 厳密に行う。時間窓での切り取り（`intersect_time` / `subtract_time`）と、
//! 約数鎖セル列（[`TemporalId`]）への最小分解（`cells` 系）もここに集約する。
//!
//! これ自体は公開せず（`pub(crate)`）、値なしの [`TemporalSet`](crate::TemporalSet) と
//! 値つきの [`TemporalMap<V>`](crate::TemporalMap) が薄い newtype ラッパとして被せる。
//! `TemporalSet = TemporalCore<()>`、`TemporalMap<V> = TemporalCore<V>`。`()` は ZST の
//! ため両者のメモリレイアウトに差はない。

use alloc::vec::Vec;

use crate::{ConflictPolicy, TemporalId};

/// 時間区間代数の中立エンジン（正規化済みセグメント列 + sweep）。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub(crate) struct TemporalCore<V> {
    /// 正規化済み（昇順・互いに素・隣接同値マージ）の `(start, end, V)`。
    segments: Vec<(u64, u64, V)>,
}

impl<V: Clone + PartialEq> TemporalCore<V> {
    /// 空。
    pub(crate) fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    /// 1つの [`TemporalId`] に値 `v` を対応させる。
    pub(crate) fn from_temporal(t: &TemporalId, v: V) -> Self {
        Self {
            segments: alloc::vec![(t.start_unixtime(), t.end_unixtime_exclusive(), v)],
        }
    }

    /// 単一セグメント `[s, e)` から作る。
    pub(crate) fn from_segment(s: u64, e: u64, v: V) -> Self {
        Self {
            segments: if s < e {
                alloc::vec![(s, e, v)]
            } else {
                Vec::new()
            },
        }
    }

    /// 空かどうか。
    pub(crate) fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// 正規化済みセグメント列を借用で返す。
    pub(crate) fn segments(&self) -> &[(u64, u64, V)] {
        &self.segments
    }

    /// 指定秒の値（二分探索）。
    pub(crate) fn value_at(&self, sec: u64) -> Option<&V> {
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
    /// 各列のカーソルを単調に進める（全体 O(n + m)、ソート不要）。
    pub(crate) fn sweep<F>(&self, other: &Self, combine: F) -> Self
    where
        F: Fn(Option<&V>, Option<&V>) -> Option<V>,
    {
        let (a, b) = (&self.segments, &other.segments);
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
        let iv = &window.segments;
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

    /// 時間窓 `window`（値なしの区間列）に含まれる時間を取り除く（値は self 由来）。
    pub(crate) fn subtract_time(&self, window: &TemporalCore<()>) -> Self {
        let iv = &window.segments;
        let mut out = Vec::new();
        let mut j = 0usize;
        for (s, e, v) in &self.segments {
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
        Self { segments: out }
    }

    /// 保持する時間セルの総数を、セルを生成せずに数える（`cells().len()` と一致）。
    ///
    /// 各セグメントを [`TemporalId::count_range`] で計上するため割当なし・
    /// O(セグメント数 × 対数) で済む（全セル展開の O(セル数) を回避）。
    pub(crate) fn count_cells(&self) -> usize {
        self.segments
            .iter()
            .map(|(s, e, _)| TemporalId::count_range(*s, *e))
            .sum()
    }

    /// 全セグメントを約数鎖セル列 `(TemporalId, V)` へ最小分解する。
    pub(crate) fn cells(&self) -> Vec<(TemporalId, V)> {
        self.segments
            .iter()
            .flat_map(|(s, e, v)| TemporalId::cells_in_range(*s, *e).map(move |c| (c, v.clone())))
            .collect()
    }

    /// [`cells`](Self::cells) の参照版（値をクローンしない）。
    pub(crate) fn cells_ref(&self) -> Vec<(TemporalId, &V)> {
        self.cells_ref_iter().collect()
    }

    /// [`cells_ref`](Self::cells_ref) の遅延イテレータ版（中間 `Vec` を作らない）。
    pub(crate) fn cells_ref_iter(&self) -> impl Iterator<Item = (TemporalId, &V)> + '_ {
        self.segments
            .iter()
            .flat_map(|(s, e, v)| TemporalId::cells_in_range(*s, *e).map(move |c| (c, v)))
    }

    /// `window` に限定したセル列を参照で返す（`(self ∩ window)` の分解）。
    pub(crate) fn cells_clipped_ref(&self, window: &TemporalId) -> Vec<(TemporalId, &V)> {
        self.cells_clipped_ref_iter(window.clone()).collect()
    }

    /// [`cells_clipped_ref`](Self::cells_clipped_ref) の遅延イテレータ版。
    ///
    /// 返すイテレータは `self`（`'a`）だけを借用する。窓は値で受け取り（`TemporalId` は
    /// 小さく `Clone` 可能）、`w0/w1` を先に取り出すため入力ライフタイムを捕捉しない
    /// ＝ 呼び出し側は同じ式内で他方を move できる。
    pub(crate) fn cells_clipped_ref_iter(
        &self,
        window: TemporalId,
    ) -> impl Iterator<Item = (TemporalId, &V)> + '_ {
        let (w0, w1) = (window.start_unixtime(), window.end_unixtime_exclusive());
        self.segments.iter().flat_map(move |(s, e, v)| {
            let cs = (*s).max(w0);
            let ce = (*e).min(w1);
            TemporalId::cells_in_range(cs, ce).map(move |c| (c, v))
        })
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
    pub(crate) fn intersection(&self, other: &Self, policy: &ConflictPolicy<V>) -> Self {
        self.sweep(other, |a, b| match (a, b) {
            (Some(a), Some(b)) => Some(policy.resolve(Some(a.clone()), b.clone())),
            _ => None,
        })
    }
}
