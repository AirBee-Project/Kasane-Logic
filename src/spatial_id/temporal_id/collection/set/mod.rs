use crate::TemporalId;
use alloc::vec::Vec;

/// [temporalId]の集合を表す型。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct TemporalSet {
    /// 互いに素・隣接非連結の `[start, end)` 。
    intervals: Vec<(u64, u64)>,
}

impl TemporalSet {
    /// 空集合を作る。
    pub fn new() -> Self {
        Self {
            intervals: Vec::new(),
        }
    }

    /// 全ての時間を表すの集合を作成する。
    pub fn whole() -> Self {
        Self {
            intervals: alloc::vec![(0, crate::Interval::WHOLE_SECONDS)],
        }
    }

    /// この集合は全ての時間を表しているかを判定する。
    pub fn is_whole(&self) -> bool {
        self.intervals.as_slice() == [(0, crate::Interval::WHOLE_SECONDS)]
    }

    /// 1つの [`TemporalId`] が覆う時間の集合を作る。
    pub fn from_temporal(t: &TemporalId) -> Self {
        Self {
            intervals: alloc::vec![(t.start_unixtime(), t.end_unixtime_exclusive())],
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

    /// 正規化済み区間列 `[start, end)` を返す（クレート内部の走査用フック）。
    pub(crate) fn intervals(&self) -> &[(u64, u64)] {
        &self.intervals
    }

    /// 指定の UNIX 秒が含まれるか（二分探索）。
    pub fn contains_unixtime(&self, sec: u64) -> bool {
        let idx = self.intervals.partition_point(|&(s, _)| s <= sec);
        idx > 0 && sec < self.intervals[idx - 1].1
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

    /// 積集合。
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

    /// 集合を約数鎖の最小セル列（[`TemporalId`]）へ分解する。
    ///
    /// 約数鎖に二進層（`Day·2^k`）があるため、どの区間も高々数百セルに収まる
    /// （ドメイン全体は単一の [`TemporalId::WHOLE`] になる）。
    pub fn cells(&self) -> Vec<TemporalId> {
        let mut out = Vec::new();
        for &(s, e) in &self.intervals {
            out.extend(TemporalId::from_range(s, e).unwrap());
        }
        out
    }

    /// `window` に限定したセル列を返す（`(self ∩ window)` の分解）。
    pub fn cells_in_window(&self, window: &TemporalId) -> Vec<TemporalId> {
        self.intersection(&Self::from_temporal(window)).cells()
    }
}

#[cfg(test)]
mod tests;
