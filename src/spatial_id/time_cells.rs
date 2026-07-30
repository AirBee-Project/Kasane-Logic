//! 絶対秒区間と、FlexTree が使う「2の冪秒のセル」との相互変換。クレート内部専用。
//!
//! 時間の値そのものは各 ID がフィールドとして持つので、ここに型は無く、
//! 秒（`u64`）とセル（`(zoom, index)`）の間を行き来する自由関数だけを置く。
//!
//! # なぜ変換が要るのか
//!
//! 仕様は時間間隔 `{i}` に**任意の秒数**を認める（`1800` 秒など）。一方 FlexTree は
//! 時間軸を2進トライとして持つため、格納できるのは `2^(35 - zoom)` 秒のセルだけである。
//! そのため木への出し入れでこの2つの表現を変換する。
//!
//! - 書き込み: 秒区間 → [`split_seconds`] で高々 `O(log 秒数)` 個のセルへ分解
//! - 読み出し: セル → [`cell_seconds_range`] で秒区間へ戻し、[`coarsest_unit`] で
//!   「その区間を表せる最も粗い `{i}`」を復元

use crate::spatial_id::zoom_level::TZoomLevel;

/// セル `(zoom, index)` が表す絶対秒区間 `[start, end)`。
///
/// 1セルの幅は `2^(TZoomLevel::MAX - zoom)` 秒。
pub(crate) const fn cell_seconds_range(zoom: u8, index: u64) -> (u64, u64) {
    let width = 1u64 << (TZoomLevel::MAX.get() - zoom);
    let start = index * width;
    (start, start + width)
}

/// 秒区間 `[start, end)` を表せる**最も粗い**単位（秒数）を返す。
///
/// 単位は `start` と区間幅の両方を割り切れなければならないので、条件を満たす最大の値は
/// `gcd(start, span)` である。固定候補（Day/Hour/Minute/Second）を総当たりする代わりに
/// これを直接求めることで、仕様が認める任意秒数の単位（`1800`、`7200`、`300` など）も
/// そのまま復元できる。
///
/// | 秒区間 | 結果 |
/// |---|---|
/// | `[0, 2^35)` | `34359738368`（全時間） |
/// | `[86400*20486, +86400)` | `86400` |
/// | `[1800*809712, +1800)` | `1800` |
/// | `[7200, 14400)` | `7200` |
///
/// 戻り値は必ず `1..=2^35` に収まる（`1 <= gcd <= span <= 2^35`）ので、
/// [`Interval::from_seconds_unchecked`](crate::Interval::from_seconds_unchecked) に渡せる。
pub(crate) const fn coarsest_unit(start: u64, span: u64) -> u64 {
    // ユークリッドの互除法。`gcd(0, b) == b` なので start == 0 でもそのまま動く。
    let (mut a, mut b) = (start, span);
    while b != 0 {
        let rem = a % b;
        a = b;
        b = rem;
    }
    a
}

/// 2つの絶対秒区間 `[start, end)` の交差。重なりが無ければ [`None`]。
pub(crate) fn intersect_seconds(a: (u64, u64), b: (u64, u64)) -> Option<(u64, u64)> {
    let start = a.0.max(b.0);
    let end = a.1.min(b.1);
    (start < end).then_some((start, end))
}

/// 絶対秒区間の差集合（`a - b`）。
///
/// `b` が `a` の内側を刳り抜く場合は前後2つ、端を削る場合は1つ、`a` を完全に覆う場合は
/// 0個を返す。重なりが無ければ `a` をそのまま1つ返す。
pub(crate) fn difference_seconds(
    a: (u64, u64),
    b: (u64, u64),
) -> impl Iterator<Item = (u64, u64)> + use<> {
    let (a_start, a_end) = a;
    let (b_start, b_end) = b;

    if b_end <= a_start || a_end <= b_start {
        return [Some(a), None].into_iter().flatten();
    }

    let head_end = b_start.min(a_end);
    let head = (a_start < head_end).then_some((a_start, head_end));

    let tail_start = b_end.max(a_start);
    let tail = (tail_start < a_end).then_some((tail_start, a_end));

    [head, tail].into_iter().flatten()
}

/// 秒区間 `[start, end)` を、区間木的に高々 `O(log 幅)` 個の2進セルへ分解する。
///
/// 空区間（`start >= end`）では何も返さない。
pub(crate) fn split_seconds(start: u64, end: u64) -> TimeCells {
    if start >= end {
        return TimeCells {
            l: 1,
            r: 0,
            cur_z: 0,
        };
    }
    split_inclusive(start, end - 1)
}

/// `[l, r]`（両端含む、1秒単位）を2進セルへ分解する。
fn split_inclusive(l: u64, r: u64) -> TimeCells {
    // 最深ズーム（1秒）から1段ずつ繰り上がるのではなく、**この区間をそのまま表せる最も粗い
    // ズーム**から始める。
    //
    // 区間 `[l, r]` をレベル `MAX - k` のインデックスへ縮めてよいのは、両端が `2^k` に
    // 揃っているとき、つまり `l` と `r + 1` がどちらも `2^k` の倍数のときに限る。
    // したがって `k = min(l の下位0の個数, (r+1) の下位0の個数)`。
    //
    // これがないと、全時間のように1セルで済む区間でも毎回35回ループする。時間を使わない
    // ビルドでも `RangeId::into_iter()` は必ずここを通るため、固定費として効く。
    let trailing = |v: u64| {
        if v == 0 {
            u64::BITS
        } else {
            v.trailing_zeros()
        }
    };
    let k = trailing(l)
        .min(trailing(r.wrapping_add(1)))
        .min(TZoomLevel::MAX.get() as u32) as u8;

    TimeCells {
        l: l >> k,
        r: r >> k,
        cur_z: (TZoomLevel::MAX.get() - k) as i8,
    }
}

/// [`split_seconds`] が返す、`(zoom, index)` の列。
///
/// `impl Iterator` ではなく名前付きの型にしてあるのは、`SingleId::into_iter` が
/// これを構造体のフィールドとして直接持てるようにするため（`Box<dyn Iterator>` を挟むと
/// 挿入1回ごとにヒープ確保が入る）。
pub(crate) struct TimeCells {
    l: u64,
    r: u64,
    cur_z: i8,
}

impl Iterator for TimeCells {
    type Item = (u8, u64);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.l > self.r {
                return None;
            }

            if self.cur_z == 0 {
                let v = self.l;
                self.l += 1;
                return Some((0, v));
            }

            let z = self.cur_z as u8;
            if self.l == self.r {
                let v = self.l;
                self.l += 1;
                return Some((z, v));
            }
            if self.l & 1 == 1 {
                let v = self.l;
                self.l += 1;
                return Some((z, v));
            }
            if self.r & 1 == 0 {
                let v = self.r;
                self.r -= 1;
                return Some((z, v));
            }
            self.l >>= 1;
            self.r >>= 1;
            self.cur_z -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Interval;
    use alloc::vec::Vec;

    /// 分解したセルは元の秒区間を隙間なく・はみ出さずに覆う。
    #[test]
    fn split_covers_every_range_exactly() {
        let cases: &[(u64, u64)] = &[
            (0, Interval::WHOLE_SECONDS),
            (0, 1),
            (1, 2),
            (3600, 7200),
            (1800 * 809_712, 1800 * 809_713),
            (1_770_000_000, 1_770_000_001),
            (5, 12),
            (7, 8),
            (86_400 * 20_486, 86_400 * 20_487),
            (1, Interval::WHOLE_SECONDS),
        ];

        for &(start, end) in cases {
            let mut ranges: Vec<_> = split_seconds(start, end)
                .map(|(z, i)| cell_seconds_range(z, i))
                .collect();
            ranges.sort_unstable();

            let mut cursor = start;
            for (s, e) in &ranges {
                assert_eq!(*s, cursor, "[{start}, {end}) でセルが連続していない");
                cursor = *e;
            }
            assert_eq!(cursor, end, "[{start}, {end}) の合計が一致しない");
        }
    }

    /// 全時間は1セルで表せるので、繰り上がりが1段も走らない。
    #[test]
    fn whole_range_is_a_single_cell() {
        let cells: Vec<_> = split_seconds(0, Interval::WHOLE_SECONDS).collect();
        assert_eq!(cells, [(0, 0)]);
    }

    #[test]
    fn empty_range_yields_nothing() {
        assert_eq!(split_seconds(5, 5).count(), 0);
        assert_eq!(split_seconds(9, 5).count(), 0);
    }

    /// セル1個の秒区間は、その幅がそのまま最も粗い単位になる
    /// （開始秒が幅の倍数なので `gcd(start, width) == width`）。
    #[test]
    fn a_single_cell_restores_its_own_width() {
        for zoom in 0..=TZoomLevel::MAX.get() {
            let max_index = (1u64 << zoom) - 1;
            for index in [0u64, 1, max_index].into_iter().filter(|i| *i <= max_index) {
                let (start, end) = cell_seconds_range(zoom, index);
                assert_eq!(coarsest_unit(start, end - start), end - start, "z={zoom}");
            }
        }
    }

    /// 固定候補では戻せなかった任意秒数の単位まで復元できる。
    #[test]
    fn coarsest_unit_restores_arbitrary_intervals() {
        for (start, end, expected) in [
            (0u64, Interval::WHOLE_SECONDS, Interval::WHOLE_SECONDS),
            (86_400 * 20_486, 86_400 * 20_487, 86_400),
            (3_600 * 491_666, 3_600 * 491_667, 3_600),
            (60 * 3, 60 * 4, 60),
            (1800 * 809_712, 1800 * 809_713, 1800),
            (7_200, 14_400, 7_200),
            (300 * 4_858_272, 300 * 4_858_273, 300),
            (1_770_000_000, 1_770_000_001, 1),
        ] {
            assert_eq!(
                coarsest_unit(start, end - start),
                expected,
                "[{start}, {end})"
            );
        }
    }

    #[test]
    fn intersect_and_difference_agree_with_the_set_operations() {
        // [3600, 7200) ∩ [5400, 9000) = [5400, 7200)
        assert_eq!(
            intersect_seconds((3600, 7200), (5400, 9000)),
            Some((5400, 7200))
        );
        assert_eq!(intersect_seconds((0, 100), (100, 200)), None);

        // 内側を刳り抜くと前後2つ。
        let pieces: Vec<_> = difference_seconds((0, 86_400), (3_600, 7_200)).collect();
        assert_eq!(pieces, [(0, 3_600), (7_200, 86_400)]);

        // 端を削ると1つ。
        let pieces: Vec<_> = difference_seconds((0, 86_400), (0, 3_600)).collect();
        assert_eq!(pieces, [(3_600, 86_400)]);

        // 完全に覆われると0個。
        assert_eq!(difference_seconds((10, 20), (0, 100)).count(), 0);

        // 重ならなければそのまま1つ。
        let pieces: Vec<_> = difference_seconds((0, 10), (50, 60)).collect();
        assert_eq!(pieces, [(0, 10)]);
    }
}
