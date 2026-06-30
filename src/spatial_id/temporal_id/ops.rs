use alloc::vec::Vec;

use crate::TemporalId;

impl TemporalId {
    /// 2つのTemporalIdの重なる時間範囲（Intersection）を計算して返す。
    ///
    /// 2つの時間区間の交差が存在し、かつ [`TemporalId`] の定義する時間間隔で
    /// 正確に表現できる場合、交差を表す新しい [`TemporalId`] を返す。
    /// 重なりがない場合や、交差が時間間隔の境界に合致しない場合は `None` を返す。
    ///
    /// # パラメーター
    ///
    /// * `other` — 交差を計算する相手の [`TemporalId`]。
    ///
    /// # 戻り値
    ///
    /// 交差を表す [`TemporalId`] が存在する場合は `Some(id)`、
    /// そうでない場合は `None` を返す。
    ///
    /// # 例
    ///
    /// 交差がある場合:
    /// ```
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::TemporalId;
    /// let id1 = TemporalId::new(3600, 5).unwrap();  // [18000, 21599]
    /// let id2 = TemporalId::new(3600, 6).unwrap();  // [21600, 24999]
    /// assert_eq!(id1.intersection(&id2), None);     // No overlap
    ///
    /// let id3 = TemporalId::new(1, 18000).unwrap(); // [18000, 18000]
    /// let inter = id1.intersection(&id3);
    /// assert!(inter.is_some());
    /// # }
    /// ```
    pub fn intersection(&self, other: &TemporalId) -> Option<TemporalId> {
        let self_start = self.start_unixstamp();
        let self_end_excl = self.end_unixtime_exclusive() as u64;
        let other_start = other.start_unixstamp();
        let other_end_excl = other.end_unixtime_exclusive() as u64;

        let inter_start = self_start.max(other_start);
        let inter_end_excl = self_end_excl.min(other_end_excl);

        if inter_start >= inter_end_excl {
            return None;
        }

        // Try to find a TemporalId that exactly represents this intersection
        for &interval in &Self::TEMPORAL_I {
            if inter_start.is_multiple_of(interval)
                && (inter_end_excl - inter_start).is_multiple_of(interval)
            {
                let t = inter_start / interval;
                if interval * (t + 1) == inter_end_excl {
                    return TemporalId::new(interval, t).ok();
                }
            }
        }

        None
    }

    /// 相手の [`TemporalId`] との差集合（self - other）を計算し、
    /// イテレータとして返す。
    ///
    /// `self` の時間範囲から `other` の時間範囲を除いた部分を計算する。
    /// 結果は0個、1個、または2個の [`TemporalId`] となる。
    ///
    /// 差集合の各要素は、元々の `self` と同じ時間間隔を持つ場合、
    /// その間隔で表現される。異なる間隔を持つ場合は、
    /// より小さい間隔への分割が行われる可能性がある。
    ///
    /// # パラメーター
    ///
    /// * `other` — `self` から除外する [`TemporalId`]。
    ///
    /// # 戻り値
    ///
    /// 差集合を表す [`TemporalId`] のイテレータ。
    ///
    /// # 例
    ///
    /// 重なりがない場合（self全体が返される）:
    /// ```
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::TemporalId;
    /// let id1 = TemporalId::new(3600, 0).unwrap();   // [0, 3599]
    /// let id2 = TemporalId::new(3600, 5).unwrap();   // [18000, 21599]
    /// let diff: Vec<_> = id1.difference(&id2).collect();
    /// assert_eq!(diff.len(), 1);
    /// assert_eq!(diff[0], id1);
    /// # }
    /// ```
    ///
    /// 左右に分かれる場合:
    /// ```
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::TemporalId;
    /// let id1 = TemporalId::new(3600, 5).unwrap();   // [18000, 21599]
    /// let id2 = TemporalId::new(1, 19800).unwrap();  // [19800, 19800]（中間の1秒）
    /// let diff: Vec<_> = id1.difference(&id2).collect();
    /// // 差集合は複数のピースに分かれる可能性がある
    /// # }
    /// ```
    ///
    /// 完全に包含される場合（空のイテレータ）:
    /// ```
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::TemporalId;
    /// let id1 = TemporalId::new(1, 19800).unwrap();  // [19800, 19800]
    /// let id2 = TemporalId::new(3600, 5).unwrap();   // [18000, 21599]
    /// let diff: Vec<_> = id1.difference(&id2).collect();
    /// assert_eq!(diff.len(), 0);
    /// # }
    /// ```
    pub fn difference(&self, other: &TemporalId) -> impl Iterator<Item = TemporalId> {
        let s0 = self.start_unixstamp();
        let s1 = self.end_unixtime_exclusive() as u64;
        let o0 = other.start_unixstamp();
        let o1 = other.end_unixtime_exclusive() as u64;

        let mut result = Vec::new();

        // 重なりなし → self をそのまま返す
        if o1 <= s0 || o0 >= s1 {
            result.push(self.clone());
            return result.into_iter();
        }

        // 左側 [s0, min(o0, s1)) と右側 [max(o1, s0), s1) を、それぞれ
        // カレンダー最小分解（[`from_range`]）で表す（方法1）。
        // 1秒展開のフォールバックは行わない（差分が膨張しない）。
        let left_end = o0.min(s1);
        if s0 < left_end
            && let Ok(cells) = Self::from_range(s0, left_end)
        {
            result.extend(cells);
        }

        let right_start = o1.max(s0);
        if right_start < s1
            && let Ok(cells) = Self::from_range(right_start, s1)
        {
            result.extend(cells);
        }

        result.into_iter()
    }

    /// 時間窓 `window` に限定した差集合 `(self ∩ window) − other` をカレンダー最小分解で返す（方法2）。
    ///
    /// `self` が WHOLE でも `window` が有界なら結果は有界になる（WHOLE 差分の膨張を防ぐ）。
    /// 集合論的に `(self ∩ window) − other = (self − other) ∩ window` と一致する。
    pub fn difference_in_window(&self, other: &TemporalId, window: &TemporalId) -> Vec<TemporalId> {
        let w0 = window.start_unixstamp();
        let w1 = window.end_unixtime_exclusive() as u64;
        let s0 = self.start_unixstamp().max(w0);
        let s1 = (self.end_unixtime_exclusive() as u64).min(w1);

        let mut result = Vec::new();
        if s0 >= s1 {
            return result;
        }

        let o0 = other.start_unixstamp();
        let o1 = other.end_unixtime_exclusive() as u64;

        // other が窓内の self と重ならない → クリップした self をそのまま分解
        if o1 <= s0 || o0 >= s1 {
            if let Ok(cells) = Self::from_range(s0, s1) {
                result = cells;
            }
            return result;
        }

        let left_end = o0.min(s1);
        if s0 < left_end
            && let Ok(cells) = Self::from_range(s0, left_end)
        {
            result.extend(cells);
        }

        let right_start = o1.max(s0);
        if right_start < s1
            && let Ok(cells) = Self::from_range(right_start, s1)
        {
            result.extend(cells);
        }

        result
    }

    /// `other` の時間範囲が `self` に完全に含まれるか（`other ⊆ self`）を判定する。
    pub fn contains(&self, other: &TemporalId) -> bool {
        self.start_unixstamp() <= other.start_unixstamp()
            && other.end_unixtime_exclusive() <= self.end_unixtime_exclusive()
    }
}

#[cfg(test)]
mod tests {
    use crate::TemporalId;
    use alloc::collections::BTreeSet;
    use alloc::vec::Vec;

    /// 1セルが覆う秒の集合 `[start, end)`。
    fn seconds(t: &TemporalId) -> BTreeSet<u64> {
        (t.start_unixstamp()..t.end_unixtime_exclusive() as u64).collect()
    }

    /// セル列が覆う秒の集合（重複は潰される）。
    fn seconds_of(cells: &[TemporalId]) -> BTreeSet<u64> {
        let mut set = BTreeSet::new();
        for c in cells {
            set.extend(c.start_unixstamp()..c.end_unixtime_exclusive() as u64);
        }
        set
    }

    /// 秒単位の長さ合計（重なり検出用）。
    fn total_len(cells: &[TemporalId]) -> u64 {
        cells
            .iter()
            .map(|c| c.end_unixtime_exclusive() as u64 - c.start_unixstamp())
            .sum()
    }

    /// [0, 7200) に収まる代表セル（入れ子・非交差・境界）。
    fn sample_cells() -> Vec<TemporalId> {
        let mut v = Vec::new();
        for t in [0u64, 1, 59, 60, 100, 3599, 3600, 7199] {
            v.push(TemporalId::new(1, t).unwrap());
        }
        for t in [0u64, 1, 59, 60, 119] {
            v.push(TemporalId::new(60, t).unwrap());
        }
        for t in [0u64, 1] {
            v.push(TemporalId::new(3600, t).unwrap());
        }
        v
    }

    /// 厳格オラクル：秒集合に展開して intersection / difference / contains を照合する。
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
        let hour = TemporalId::new(3600, 0).unwrap(); // [0, 3600)
        let min = TemporalId::new(60, 0).unwrap(); // [0, 60)
        let d: Vec<_> = hour.difference(&min).collect();
        assert_eq!(d.len(), 59, "59個の分セルのはず");
        assert!(d.iter().all(|c| c.i() == 60), "全て i=60（分単位）");
        assert_eq!(seconds_of(&d), (60u64..3600).collect());
    }

    /// WHOLE − 1分 を「窓（その1時間）」に限定すると有界（59分）になる。
    #[test]
    fn whole_minus_minute_in_window_is_bounded() {
        let whole = TemporalId::WHOLE;
        let min = TemporalId::new(60, 600).unwrap(); // [36000, 36060)
        let hour = TemporalId::new(3600, 10).unwrap(); // [36000, 39600)
        let d = whole.difference_in_window(&min, &hour);
        assert_eq!(d.len(), 59);
        assert!(d.iter().all(|c| c.i() == 60));
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
            TemporalId::new(3600, 0).unwrap(),
            TemporalId::new(3600, 1).unwrap(),
            TemporalId::new(60, 30).unwrap(),
            TemporalId::WHOLE,
        ];
        for a in &cells {
            for b in &cells {
                for w in &windows {
                    let got = seconds_of(&a.difference_in_window(b, w));
                    // オラクル: (seconds(a) − seconds(b)) ∩ window
                    // window は WHOLE だと巨大なので述語で判定（展開しない）。
                    let in_w = |s: &u64| {
                        w.start_unixstamp() <= *s && (*s as u128) < w.end_unixtime_exclusive()
                    };
                    let sa = seconds(a);
                    let sb = seconds(b);
                    let exp: BTreeSet<u64> = sa.difference(&sb).copied().filter(in_w).collect();
                    assert_eq!(got, exp, "window diff a={a:?} b={b:?} w={w:?}");
                }
            }
        }
    }
}
