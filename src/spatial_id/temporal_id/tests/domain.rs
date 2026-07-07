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
            v.push(TemporalId::new(1_u64, t).unwrap());
        }
        for t in [0u64, 1, 59, 60, 119] {
            v.push(TemporalId::new(60_u64, t).unwrap());
        }
        for t in [0u64, 1] {
            v.push(TemporalId::new(3600_u64, t).unwrap());
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
        let hour = TemporalId::new(3600_u64, 0).unwrap(); // [0, 3600)
        let min = TemporalId::new(60_u64, 0).unwrap(); // [0, 60)
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
        let min = TemporalId::new(60_u64, 600).unwrap(); // [36000, 36060)
        let d: Vec<_> = whole.difference(&min).collect();
        // 二進層のおかげで爆発しない（左側 + 右側で高々数百）。
        assert!(d.len() < 400, "cells = {}", d.len());
        // 被覆の検証（秒展開せず区間で照合）:
        // 合計長 = 全長 − 60秒、穴の周辺だけピンポイントで欠けている。
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
        let min = TemporalId::new(60_u64, 600).unwrap(); // [36000, 36060)
        let hour = TemporalId::new(3600_u64, 10).unwrap(); // [36000, 39600)
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
            TemporalId::new(3600_u64, 0).unwrap(),
            TemporalId::new(3600_u64, 1).unwrap(),
            TemporalId::new(60_u64, 30).unwrap(),
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

    /// 時間 `[0, WHOLE_SECONDS)` の境界検証。
    #[test]
    fn domain_boundary() {
        // 終端を超えるIDは構築できない
        assert!(TemporalId::new(1_u64, Interval::WHOLE_SECONDS).is_err());
        // 最終秒 [WHOLE_SECONDS-1, WHOLE_SECONDS) は有効で、WHOLE に含まれる
        let last = TemporalId::new(1_u64, Interval::WHOLE_SECONDS - 1).unwrap();
        assert_eq!(last.end_unixtime_exclusive(), Interval::WHOLE_SECONDS);
        assert!(TemporalId::WHOLE.contains(&last));
        // WHOLE 自身の終端は終端
        assert_eq!(
            TemporalId::WHOLE.end_unixtime_exclusive(),
            Interval::WHOLE_SECONDS
        );
        // WHOLE の間隔は全長そのもの（u64::MAX は約数鎖に無い）
        assert_eq!(
            TemporalId::new(Interval::WHOLE_SECONDS, 0).unwrap(),
            TemporalId::WHOLE
        );
        assert!(TemporalId::new(u64::MAX, 0).is_err());
    }
}
