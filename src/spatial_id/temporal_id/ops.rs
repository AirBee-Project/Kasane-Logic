use alloc::vec::Vec;
use core::ops::{BitAnd, Sub};

use crate::TemporalId;

impl TemporalId {
    /// 2つのTemporalIdの重なる時間範囲（Intersection）を返す。
    /// # 例
    ///
    /// ```
    /// # use kasane_logic::TemporalId;
    /// let id1 = TemporalId::from_seconds(3600, 5).unwrap();  // [18000, 21600)
    /// let id2 = TemporalId::from_seconds(3600, 6).unwrap();  // [21600, 25200)
    /// assert_eq!(id1.intersection(&id2), None);     // 重なりなし
    ///
    /// let id3 = TemporalId::from_seconds(1, 18000).unwrap(); // [18000, 18001)
    /// assert_eq!(id1.intersection(&id3), Some(id3.clone()));
    /// ```
    pub fn intersection(&self, other: &TemporalId) -> Option<TemporalId> {
        if self.contains(other) {
            Some(other.clone())
        } else if other.contains(self) {
            Some(self.clone())
        } else {
            None
        }
    }

    /// 相手の [`TemporalId`] との差集合（self - other）を計算し、イテレータとして返す。
    /// # 例
    ///
    /// 重なりがない場合（self全体が返される）:
    /// ```
    /// # use kasane_logic::TemporalId;
    /// let id1 = TemporalId::from_seconds(3600, 0).unwrap();   // [0, 3600)
    /// let id2 = TemporalId::from_seconds(3600, 5).unwrap();   // [18000, 21600)
    /// let diff: Vec<_> = id1.difference(&id2).collect();
    /// assert_eq!(diff.len(), 1);
    /// assert_eq!(diff[0], id1);
    /// ```
    ///
    /// 完全に包含される場合（空のイテレータ）:
    /// ```
    /// # use kasane_logic::TemporalId;
    /// let id1 = TemporalId::from_seconds(1, 19800).unwrap();  // [19800, 19801)
    /// let id2 = TemporalId::from_seconds(3600, 5).unwrap();   // [18000, 21600)
    /// let diff: Vec<_> = id1.difference(&id2).collect();
    /// assert_eq!(diff.len(), 0);
    /// ```
    pub fn difference(&self, other: &TemporalId) -> impl Iterator<Item = TemporalId> {
        let s0 = self.start_unixtime();
        let s1 = self.end_unixtime_exclusive();
        let o0 = other.start_unixtime();
        let o1 = other.end_unixtime_exclusive();

        let mut result = Vec::new();

        if o1 <= s0 || o0 >= s1 {
            result.push(self.clone());
            return result.into_iter();
        }

        let left_end = o0.min(s1);
        if s0 < left_end {
            result.extend(Self::from_range(s0, left_end).unwrap());
        }

        let right_start = o1.max(s0);
        if right_start < s1 {
            result.extend(Self::from_range(right_start, s1).unwrap());
        }

        result.into_iter()
    }

    /// 時間窓 `window` に限定した差集合 `(self ∩ window) − other` を最小分解で返す。
    ///
    /// 結果を `window` の範囲に切り詰めたい場合に使う。
    /// 集合論的に `(self ∩ window) − other = (self − other) ∩ window` と一致する。
    pub fn difference_clipped(&self, other: &TemporalId, window: &TemporalId) -> Vec<TemporalId> {
        let w0 = window.start_unixtime();
        let w1 = window.end_unixtime_exclusive();
        let s0 = self.start_unixtime().max(w0);
        let s1 = self.end_unixtime_exclusive().min(w1);

        let mut result = Vec::new();
        if s0 >= s1 {
            return result;
        }

        let o0 = other.start_unixtime();
        let o1 = other.end_unixtime_exclusive();

        if o1 <= s0 || o0 >= s1 {
            return Self::from_range(s0, s1).unwrap();
        }

        let left_end = o0.min(s1);
        if s0 < left_end {
            result.extend(Self::from_range(s0, left_end).unwrap());
        }

        let right_start = o1.max(s0);
        if right_start < s1 {
            result.extend(Self::from_range(right_start, s1).unwrap());
        }

        result
    }

    /// `other` の時間範囲が `self` に完全に含まれるか（`other ⊆ self`）を判定する。
    pub fn contains(&self, other: &TemporalId) -> bool {
        self.start_unixtime() <= other.start_unixtime()
            && other.end_unixtime_exclusive() <= self.end_unixtime_exclusive()
    }
}

impl BitAnd for &TemporalId {
    type Output = Option<TemporalId>;
    fn bitand(self, rhs: Self) -> Self::Output {
        self.intersection(rhs)
    }
}

impl Sub for &TemporalId {
    type Output = Vec<TemporalId>;
    fn sub(self, rhs: Self) -> Self::Output {
        self.difference(rhs).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Interval, TemporalId};
    use alloc::collections::BTreeSet;
    use alloc::vec::Vec;

    /// 1セルが覆う秒の集合 `[start, end)`。
    fn seconds(t: &TemporalId) -> BTreeSet<u64> {
        (t.start_unixtime()..t.end_unixtime_exclusive()).collect()
    }

    /// セル列が覆う秒の集合（重複は潰される）。
    fn seconds_of(cells: &[TemporalId]) -> BTreeSet<u64> {
        let mut set = BTreeSet::new();
        for c in cells {
            set.extend(c.start_unixtime()..c.end_unixtime_exclusive());
        }
        set
    }

    /// 秒単位の長さ合計（重なり検出用）。
    fn total_len(cells: &[TemporalId]) -> u64 {
        cells
            .iter()
            .map(|c| c.end_unixtime_exclusive() - c.start_unixtime())
            .sum()
    }

    /// [0, 7200) に収まる代表セル（入れ子・非交差・境界）。
    fn sample_cells() -> Vec<TemporalId> {
        let mut v = Vec::new();
        for t in [0u64, 1, 59, 60, 100, 3599, 3600, 7199] {
            v.push(TemporalId::from_seconds(1, t).unwrap());
        }
        for t in [0u64, 1, 59, 60, 119] {
            v.push(TemporalId::from_seconds(60, t).unwrap());
        }
        for t in [0u64, 1] {
            v.push(TemporalId::from_seconds(3600, t).unwrap());
        }
        v
    }

    /// 厳格正解：秒集合に展開して intersection / difference / contains を照合する。
    #[test]
    fn second_set_oracle() {
        let cells = sample_cells();
        for a in &cells {
            let sa = seconds(a);
            for b in &cells {
                let sb = seconds(b);

                // intersection: 1セル or None、かつ秒集合が厳密一致
                let inter = a.intersection(b);
                let exp_inter: BTreeSet<u64> = sa.intersection(&sb).copied().collect();
                match &inter {
                    Some(i) => assert_eq!(seconds(i), exp_inter, "inter {a:?} ∩ {b:?}"),
                    None => assert!(
                        exp_inter.is_empty(),
                        "inter None だが重なりあり {a:?} {b:?}"
                    ),
                }

                // difference: 被覆が厳密一致、ピース同士は非交差
                let diff: Vec<TemporalId> = a.difference(b).collect();
                let got = seconds_of(&diff);
                let exp_diff: BTreeSet<u64> = sa.difference(&sb).copied().collect();
                assert_eq!(got, exp_diff, "diff {a:?} − {b:?}");
                assert_eq!(
                    total_len(&diff) as usize,
                    got.len(),
                    "diff ピースが重複 {a:?} − {b:?}"
                );

                // contains
                assert_eq!(a.contains(b), sb.is_subset(&sa), "contains {a:?} ⊇ {b:?}");
            }
        }
    }

    /// 1時間 − 1分 が「59個の分セル」（秒断片でない）になる。
    #[test]
    fn hour_minus_minute_is_59_minute_cells() {
        let hour = TemporalId::from_seconds(3600, 0).unwrap(); // [0, 3600)
        let min = TemporalId::from_seconds(60, 0).unwrap(); // [0, 60)
        let d: Vec<_> = hour.difference(&min).collect();
        assert_eq!(d.len(), 59, "59個の分セルのはず");
        assert!(
            d.iter().all(|c| c.i() == Interval::Minute),
            "全て i=60（分単位）"
        );
        assert_eq!(seconds_of(&d), (60u64..3600).collect());
    }

    /// WHOLE − 1分 が有界個（対数オーダー）のセルで正確に表現される。
    #[test]
    fn whole_minus_minute_is_bounded() {
        let whole = TemporalId::WHOLE;
        let min = TemporalId::from_seconds(60, 600).unwrap(); // [36000, 36060)
        let d: Vec<_> = whole.difference(&min).collect();
        // 二進層のおかげで爆発しない（左側 + 右側で高々数百）。
        assert!(d.len() < 400, "cells = {}", d.len());
        // 被覆の検証（秒展開せず区間で照合）:
        // 合計長 = ドメイン全長 − 60秒、穴の周辺だけピンポイントで欠けている。
        assert_eq!(total_len(&d), Interval::WHOLE_SECONDS - 60);
        assert!(
            d.iter()
                .all(|c| { c.end_unixtime_exclusive() <= 36000 || c.start_unixtime() >= 36060 })
        );
        // ピースは互いに素・整列済みの分解であることを軽く確認
        let mut cover = 0u64;
        let mut cells = d.clone();
        cells.sort_by_key(|c| c.start_unixtime());
        for c in &cells {
            assert!(c.start_unixtime() >= cover, "overlap");
            cover = c.end_unixtime_exclusive();
        }
    }

    /// WHOLE − 1分 を「窓（その1時間）」に限定すると 59分になる。
    #[test]
    fn whole_minus_minute_clipped_is_bounded() {
        let whole = TemporalId::WHOLE;
        let min = TemporalId::from_seconds(60, 600).unwrap(); // [36000, 36060)
        let hour = TemporalId::from_seconds(3600, 10).unwrap(); // [36000, 39600)
        let d = whole.difference_clipped(&min, &hour);
        assert_eq!(d.len(), 59);
        assert!(d.iter().all(|c| c.i() == Interval::Minute));
        let exp: BTreeSet<u64> = (36000u64..39600)
            .filter(|s| !(36000..36060).contains(s))
            .collect();
        assert_eq!(seconds_of(&d), exp);
    }

    /// `(self ∩ window) − other == (self − other) ∩ window`（窓と差分の整合）。
    #[test]
    fn window_difference_matches_clip_then_difference() {
        let cells = sample_cells();
        let windows = [
            TemporalId::from_seconds(3600, 0).unwrap(),
            TemporalId::from_seconds(3600, 1).unwrap(),
            TemporalId::from_seconds(60, 30).unwrap(),
        ];
        for a in &cells {
            for b in &cells {
                for w in &windows {
                    let got = seconds_of(&a.difference_clipped(b, w));
                    let in_w =
                        |s: &u64| w.start_unixtime() <= *s && *s < w.end_unixtime_exclusive();
                    let sa = seconds(a);
                    let sb = seconds(b);
                    let exp: BTreeSet<u64> = sa.difference(&sb).copied().filter(in_w).collect();
                    assert_eq!(got, exp, "window diff a={a:?} b={b:?} w={w:?}");
                }
            }
        }
    }

    /// 時間ドメイン `[0, WHOLE_SECONDS)` の境界検証。
    #[test]
    fn domain_boundary() {
        // 終端を超えるIDは構築できない
        assert!(TemporalId::from_seconds(1, Interval::WHOLE_SECONDS).is_err());
        // 最終秒 [WHOLE_SECONDS-1, WHOLE_SECONDS) は有効で、WHOLE に含まれる
        let last = TemporalId::from_seconds(1, Interval::WHOLE_SECONDS - 1).unwrap();
        assert_eq!(last.end_unixtime_exclusive(), Interval::WHOLE_SECONDS);
        assert!(TemporalId::WHOLE.contains(&last));
        // WHOLE 自身の終端はドメイン終端
        assert_eq!(
            TemporalId::WHOLE.end_unixtime_exclusive(),
            Interval::WHOLE_SECONDS
        );
        // WHOLE の間隔はドメイン全長そのもの（u64::MAX は約数鎖に無い）
        assert_eq!(
            TemporalId::from_seconds(Interval::WHOLE_SECONDS, 0).unwrap(),
            TemporalId::WHOLE
        );
        assert!(TemporalId::from_seconds(u64::MAX, 0).is_err());
    }
}
