//! [`TemporalSet`]: カレンダー時間の集合（1次元）。
//!
//! 内部表現は **正規化済み**（昇順・互いに素・隣接非連結）の半開区間 `[start, end)`（秒）。
//! 区間代数で union / intersection / difference を厳密に行い、出力（[`cells`](TemporalSet::cells)）は
//! [`TemporalId`] のカレンダーセル列へ最小分解する（時/分/日などの慣例単位を保つ）。
//!
//! これは空間主体統合（FlexTree の葉に持たせる時間構造）の土台となる独立部品で、
//! コレクション本体には依存しない。

use alloc::vec::Vec;

use crate::TemporalId;

/// WHOLE（全時間）区間の排他的終端。
const WHOLE_END: u64 = u64::MAX;

/// カレンダー時間の集合。互いに素な半開区間 `[start, end)` の正規化列で表す。
///
/// 正規化済みなので比較は正準（同じ集合 ⇔ 同じ `intervals`）。`CellValue` として
/// コレクションの値型に使える。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct TemporalSet {
    /// 正規化済み（昇順・互いに素・隣接非連結）の `[start, end)` 列。
    intervals: Vec<(u64, u64)>,
}

impl TemporalSet {
    /// 空集合を作る。
    pub fn new() -> Self {
        Self {
            intervals: Vec::new(),
        }
    }

    /// 全時間（WHOLE）の集合を作る。
    pub fn whole() -> Self {
        Self {
            intervals: alloc::vec![(0, WHOLE_END)],
        }
    }

    /// 1つの [`TemporalId`] が覆う時間の集合を作る。
    pub fn from_temporal(t: &TemporalId) -> Self {
        let s = t.start_unixstamp();
        let e = t.end_unixtime_exclusive().min(WHOLE_END as u128) as u64;
        if s >= e {
            Self::new()
        } else {
            Self {
                intervals: alloc::vec![(s, e)],
            }
        }
    }

    /// [`TemporalId`] を集合へ追加する（union）。
    pub fn insert(&mut self, t: &TemporalId) {
        *self = self.union(&Self::from_temporal(t));
    }

    /// 空かどうか。
    pub fn is_empty(&self) -> bool {
        self.intervals.is_empty()
    }

    /// 指定の UNIX 秒が含まれるか。
    pub fn contains_unixtime(&self, sec: u64) -> bool {
        self.intervals.iter().any(|&(s, e)| s <= sec && sec < e)
    }

    /// `t` の時間範囲が完全に含まれるか（`t ⊆ self`）。
    pub fn contains(&self, t: &TemporalId) -> bool {
        Self::from_temporal(t).difference(self).is_empty()
    }

    /// 正規化（昇順ソート → 重なり/隣接をマージ → 空区間を除去）。
    fn normalize(mut v: Vec<(u64, u64)>) -> Vec<(u64, u64)> {
        v.sort_unstable();
        let mut out: Vec<(u64, u64)> = Vec::new();
        for (s, e) in v {
            if s >= e {
                continue;
            }
            if let Some(last) = out.last_mut()
                && s <= last.1
            {
                last.1 = last.1.max(e);
                continue;
            }
            out.push((s, e));
        }
        out
    }

    /// 和集合。
    pub fn union(&self, other: &Self) -> Self {
        let mut v = self.intervals.clone();
        v.extend_from_slice(&other.intervals);
        Self {
            intervals: Self::normalize(v),
        }
    }

    /// 積集合（2ポインタ走査。入力が正規化済みなら出力も正規化済み）。
    pub fn intersection(&self, other: &Self) -> Self {
        let (a, b) = (&self.intervals, &other.intervals);
        let mut out = Vec::new();
        let (mut i, mut j) = (0usize, 0usize);
        while i < a.len() && j < b.len() {
            let s = a[i].0.max(b[j].0);
            let e = a[i].1.min(b[j].1);
            if s < e {
                out.push((s, e));
            }
            if a[i].1 < b[j].1 {
                i += 1;
            } else {
                j += 1;
            }
        }
        Self { intervals: out }
    }

    /// 差集合 `self - other`。
    pub fn difference(&self, other: &Self) -> Self {
        let b = &other.intervals;
        let mut out = Vec::new();
        let mut j = 0usize;
        for &(a_s, a_e) in &self.intervals {
            let mut cur = a_s;
            // a_s より前で終わる B 区間を読み飛ばす（以降の A とも重ならない）。
            while j < b.len() && b[j].1 <= cur {
                j += 1;
            }
            let mut k = j;
            while k < b.len() && b[k].0 < a_e {
                let (b_s, b_e) = b[k];
                if b_s > cur {
                    out.push((cur, b_s));
                }
                if b_e > cur {
                    cur = b_e;
                }
                if cur >= a_e {
                    break;
                }
                k += 1;
            }
            if cur < a_e {
                out.push((cur, a_e));
            }
        }
        Self {
            intervals: Self::normalize(out),
        }
    }

    /// 集合をカレンダー最小セル列（[`TemporalId`]）へ分解する。
    /// WHOLE 区間は単一の [`TemporalId::WHOLE`] として返す（巨大分解を避ける）。
    pub fn cells(&self) -> Vec<TemporalId> {
        let mut out = Vec::new();
        for &(s, e) in &self.intervals {
            if s == 0 && e == WHOLE_END {
                out.push(TemporalId::WHOLE);
            } else if let Ok(cells) = TemporalId::from_range(s, e) {
                out.extend(cells);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::TemporalSet;
    use crate::TemporalId;
    use alloc::collections::BTreeSet;
    use alloc::vec::Vec;

    /// 集合が覆う秒（有界ドメイン前提。WHOLE は使わない）。
    fn secs(set: &TemporalSet) -> BTreeSet<u64> {
        let mut s = BTreeSet::new();
        for &(a, b) in &set.intervals {
            s.extend(a..b);
        }
        s
    }

    /// `(i, t)` のセル列から集合を構築。
    fn build(cells: &[(u64, u64)]) -> TemporalSet {
        let mut set = TemporalSet::new();
        for &(i, t) in cells {
            set.insert(&TemporalId::new(i, t).unwrap());
        }
        set
    }

    /// [0, 7200) に収まる代表集合（重なり・隣接・入れ子）。
    fn sample_sets() -> Vec<TemporalSet> {
        alloc::vec![
            TemporalSet::new(),
            build(&[(60, 0)]),
            build(&[(60, 0), (60, 2)]),
            build(&[(3600, 0)]),
            build(&[(3600, 0), (60, 60)]), // 隣接 → [0,3660)
            build(&[(1, 0), (1, 1), (1, 3)]),
            build(&[(60, 30), (3600, 1)]),
            build(&[(3600, 0), (3600, 1)]), // [0,7200)
        ]
    }

    /// 正規化不変条件: 昇順・互いに素・隣接非連結。
    fn assert_normalized(set: &TemporalSet) {
        for w in set.intervals.windows(2) {
            assert!(w[0].1 < w[1].0, "not normalized: {:?}", set.intervals);
        }
        for &(s, e) in &set.intervals {
            assert!(s < e, "empty interval: {:?}", set.intervals);
        }
    }

    /// 厳格オラクル: 秒集合に展開して union/intersection/difference/contains を照合。
    #[test]
    fn set_algebra_oracle() {
        let sets = sample_sets();
        for a in &sets {
            let sa = secs(a);
            for b in &sets {
                let sb = secs(b);

                let u = a.union(b);
                assert_eq!(secs(&u), sa.union(&sb).copied().collect());
                assert_normalized(&u);

                let i = a.intersection(b);
                assert_eq!(secs(&i), sa.intersection(&sb).copied().collect());
                assert_normalized(&i);

                let d = a.difference(b);
                assert_eq!(secs(&d), sa.difference(&sb).copied().collect());
                assert_normalized(&d);
            }
        }
    }

    /// `cells()` が被覆を保つ（往復: 集合 → セル列 → 集合 で秒集合が一致）。
    #[test]
    fn cells_roundtrip_preserves_coverage() {
        for set in sample_sets() {
            let mut rebuilt = TemporalSet::new();
            for c in set.cells() {
                rebuilt.insert(&c);
            }
            assert_eq!(secs(&set), secs(&rebuilt), "cells roundtrip mismatch");
        }
    }

    /// contains の照合（秒の部分集合判定と一致）。
    #[test]
    fn contains_oracle() {
        let sets = sample_sets();
        let probes = [
            TemporalId::new(60, 0).unwrap(),
            TemporalId::new(60, 1).unwrap(),
            TemporalId::new(3600, 0).unwrap(),
            TemporalId::new(1, 3600).unwrap(),
        ];
        for set in &sets {
            let s = secs(set);
            for p in &probes {
                let ps: BTreeSet<u64> =
                    (p.start_unixstamp()..p.end_unixtime_exclusive() as u64).collect();
                assert_eq!(
                    set.contains(p),
                    ps.is_subset(&s),
                    "contains {set:?} ⊇ {p:?}"
                );
            }
        }
    }

    /// WHOLE の扱い: cells() は単一 WHOLE、WHOLE − 1時間 は2区間。
    #[test]
    fn whole_handling() {
        let w = TemporalSet::whole();
        assert_eq!(w.cells(), alloc::vec![TemporalId::WHOLE]);

        let hour = TemporalSet::from_temporal(&TemporalId::new(3600, 10).unwrap());
        let d = w.difference(&hour);
        assert_eq!(d.intervals.len(), 2, "WHOLE − 1時間 = 前後2区間");
        assert!(!d.contains_unixtime(36000)); // 穴の中
        assert!(d.contains_unixtime(35999)); // 穴の直前
        assert!(d.contains_unixtime(39600)); // 穴の直後
    }
}
