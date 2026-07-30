#[cfg(feature = "temporal_id")]
use alloc::string::ToString;
#[cfg(feature = "temporal_id")]
use core::{fmt, str::FromStr};

#[cfg(feature = "temporal_id")]
use crate::{
    Interval, SpatialIdError, TemporalSegment,
    error::Error,
    spatial_id::{
        helpers::{IntoRange, format_dimension},
        temporal_id::zoom_level::TZoomLevel,
    },
};

/// 時間区間を表す、人間に読みやすい公開表現。
///
/// [`FlexId`](crate::FlexId)/[`SingleId`](crate::SingleId)/[`RangeId`](crate::RangeId)の
/// いずれも [`SpatialId::temporal`](crate::SpatialId::temporal)/
/// [`SingleId::with_temporal`](crate::SingleId::with_temporal)を通じてこの1つの型だけを
/// やり取りする。`RangeId.f/x/y`が「単位（ズームレベル）＋範囲」であるのと同じ形で、時間の単位
/// （[`Interval`]）とその単位でのインデックス範囲 `[min, max]`（両端含む）を保持する。
///
/// FlexTree内部（`FlexId`/`SingleId`が実際に木へ格納する形）は2の冪秒のセルを前提とする2進トライだが、
/// `Interval`（Day/Hour/Minute/Second）の秒数は2の冪とは限らない。そのため`FlexId`/`SingleId`へ
/// 付与する際は、区間木的な分解（クレート内部専用）によりちょうど1個の2進セルに一致する場合だけ
/// 受理される（`FlexId`/`SingleId`は「点」なので、複数セルにまたがる範囲は付与できない）。
#[cfg(feature = "temporal_id")]
#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct TemporalId {
    interval: Interval,
    t: [u64; 2],
}

#[cfg(feature = "temporal_id")]
impl TemporalId {
    /// 全時間を表す定数。
    pub const WHOLE: Self = TemporalId {
        interval: Interval::WHOLE,
        t: [0, 0],
    };

    /// このインスタンスが全時間を表す特別な値（[`WHOLE`](Self::WHOLE)）であるかを判定する。
    pub fn is_whole(&self) -> bool {
        self.interval == Interval::WHOLE && self.t == [0, 0]
    }

    /// 指定した単位・範囲から [`TemporalId`] を構築する。
    ///
    /// `interval`は[`Interval`]、または秒数を表す`impl Into<i64>`（無注釈の整数リテラルを含む）の
    /// どちらでも渡せる（[`FlexId::new`](crate::FlexId::new)等の`impl Into<u8>`によるズームレベル
    /// 指定と同じ考え方）。`u64`ではなく`i64`を受け取るのは、無注釈の整数リテラルの既定型が`i32`で
    /// あり、`i32`は`Into<i64>`は満たすが`Into<u64>`は満たさないため（符号あり→符号なしの暗黙変換は
    /// 存在しない）。`60`のような素のリテラルをそのまま渡せるようにする目的で`i64`を選んでいる。
    /// 負の値や[`Interval`]の候補（Whole/Day/Hour/Minute/Second）に一致しない秒数を渡した場合は
    /// [`SpatialIdError::TIntervalError`]を返す。
    ///
    /// `t`は`[min, max]`のような`[u64;2]`、または両端が等しい単一の`u64`値のどちらでも渡せる
    /// （[`RangeId::new`](crate::RangeId::new)のf/x/y引数と同じ考え方）。自動的に昇順へ
    /// 並び替えられる。範囲が表す絶対秒区間が[`Interval::WHOLE_SECONDS`]を超える場合は
    /// [`Error`]を返す。
    pub fn new(interval: impl Into<i64>, t: impl IntoRange<u64>) -> Result<Self, Error> {
        let seconds: i64 = interval.into();
        let seconds =
            u64::try_from(seconds).map_err(|_| SpatialIdError::TIntervalError { i: 0 })?;
        let interval = Interval::new(seconds)?;
        let mut t = t.into_range();
        if t[0] > t[1] {
            t.swap(0, 1);
        }

        let unit = interval.seconds();
        let end_exclusive = t[1]
            .checked_add(1)
            .and_then(|v| v.checked_mul(unit))
            .ok_or(SpatialIdError::TOutOfRange { i: unit, t: t[1] })?;

        if end_exclusive > Interval::WHOLE_SECONDS {
            return Err(SpatialIdError::TOutOfRange { i: unit, t: t[1] }.into());
        }

        Ok(Self { interval, t })
    }

    /// 検証を行わずに [`TemporalId`] を構築する。
    ///
    /// # Safety
    /// 呼び出し側は `(t[1]+1) * interval.seconds() <= Interval::WHOLE_SECONDS` を保証しなければならない。
    pub unsafe fn new_unchecked(interval: Interval, t: [u64; 2]) -> Self {
        Self { interval, t }
    }

    /// この時間区間の単位を取得する。
    pub fn interval(&self) -> Interval {
        self.interval
    }

    /// この時間区間のインデックス範囲 `[min, max]`（両端含む）を取得する。
    pub fn t(&self) -> [u64; 2] {
        self.t
    }

    /// この値が表す絶対秒区間 `[start, end)` を返す。
    pub fn seconds_range(&self) -> (u64, u64) {
        let unit = self.interval.seconds();
        (self.t[0] * unit, (self.t[1] + 1) * unit)
    }

    /// 絶対秒区間 `[start, end)` を、最も粗く一致する[`Interval`]で表した [`TemporalId`] を作る。
    ///
    /// [`Interval::coarse_to_fine`]の順に試すため、`[0, 86400)`なら`Day`、`[0, 3600)`なら`Hour`の
    /// ように「きれいな」単位が優先される。どれにも一致しない区間は
    /// [`Interval::SECOND`]（1秒は常に割り切れる）へ落ちるので、この関数は必ず成功する。
    fn from_seconds_range(start: u64, end: u64) -> Self {
        debug_assert!(start < end, "from_seconds_range は非空区間のみを受け付ける");
        let span = end - start;

        for interval in Interval::coarse_to_fine() {
            let unit = interval.seconds();
            if start.is_multiple_of(unit) && span.is_multiple_of(unit) {
                return TemporalId {
                    interval,
                    t: [start / unit, end / unit - 1],
                };
            }
        }

        unreachable!("Interval::SECOND (1秒) は常に割り切れるため、ここには到達しない")
    }

    /// 2つの [`TemporalId`] が重なっている時間区間を返す。重なりが無ければ [`None`]。
    ///
    /// 双方を絶対秒区間へ直して交差を取るため、`interval`が異なっていても（例: 1時間単位と
    /// 30分単位）正しく比較できる。結果がどちらかの被演算子と同じ区間になる場合はその
    /// [`Interval`]表記をそのまま保ち、そうでなければ最も粗い表記へ正規化する。
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        let (s1, e1) = self.seconds_range();
        let (s2, e2) = other.seconds_range();

        let start = s1.max(s2);
        let end = e1.min(e2);
        if start >= end {
            return None;
        }

        if (start, end) == (s1, e1) {
            Some(self.clone())
        } else if (start, end) == (s2, e2) {
            Some(other.clone())
        } else {
            Some(Self::from_seconds_range(start, end))
        }
    }

    /// 相手の [`TemporalId`] との差集合（`self - other`）を返す。
    ///
    /// `other`が`self`の内側を刳り抜く場合は前後2つ、端を削る場合は1つ、`self`を完全に覆う場合は
    /// 0個の [`TemporalId`] を返す。
    pub fn difference(&self, other: &Self) -> impl Iterator<Item = Self> {
        let (s1, e1) = self.seconds_range();
        let (s2, e2) = other.seconds_range();

        // 重なりが無ければ self をそのまま（表記も保ったまま）返す。
        if e2 <= s1 || e1 <= s2 {
            return [Some(self.clone()), None].into_iter().flatten();
        }

        let head_end = s2.min(e1);
        let head = (s1 < head_end).then(|| Self::from_seconds_range(s1, head_end));

        let tail_start = e2.max(s1);
        let tail = (tail_start < e1).then(|| Self::from_seconds_range(tail_start, e1));

        [head, tail].into_iter().flatten()
    }

    /// この [`TemporalId`] を、絶対秒区間へ変換した上で、FlexTree内部の生の2進セル
    /// （[`TemporalSegment`]、クレート内部専用）の列へ分解する。クレート内部専用（`pub(crate)`）。
    ///
    /// [`interval()`](Self::interval)の秒数は2の冪であるとは限らない（Day/Hour/Minuteはいずれも
    /// 2の冪ではない）ため、区間木的な分解（`SegmentIter64`）により高々`O(log 秒数)`個の2進セルへ
    /// 分解される。
    pub(crate) fn segments(&self) -> impl Iterator<Item = TemporalSegment> {
        let unit = self.interval.seconds();
        let start = self.t[0] * unit;
        let end_inclusive = (self.t[1] + 1) * unit - 1;

        split_t([start, end_inclusive])
            .map(|(zoom, index)| unsafe { TemporalSegment::new_unchecked(zoom, index) })
    }
}

/// 生の2進セル（[`TemporalSegment`]）1個を、被覆する絶対秒区間として最も粗く一致する
/// [`Interval`]ラベルで表した [`TemporalId`] へ変換する。クレート内部専用。
#[cfg(feature = "temporal_id")]
impl From<&TemporalSegment> for TemporalId {
    fn from(cell: &TemporalSegment) -> Self {
        let (start, end) = cell.seconds_range();
        Self::from_seconds_range(start, end)
    }
}

/// `[start, end]`（両端含む、1秒単位）を、区間木的に高々`O(log (end-start))`個の
/// 2進区間へ分解する（`RangeId::convert::SegmentIter`と同じアルゴリズムの64bit版）。
#[cfg(feature = "temporal_id")]
fn split_t(range: [u64; 2]) -> impl Iterator<Item = (u8, u64)> {
    let [l, r] = range;
    SegmentIter64 {
        l,
        r,
        cur_z: TZoomLevel::MAX.get() as i8,
    }
}

#[cfg(feature = "temporal_id")]
struct SegmentIter64 {
    l: u64,
    r: u64,
    cur_z: i8,
}

#[cfg(feature = "temporal_id")]
impl Iterator for SegmentIter64 {
    type Item = (u8, u64); // (zoom, index)

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

#[cfg(feature = "temporal_id")]
impl fmt::Display for TemporalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.interval, format_dimension(self.t))
    }
}

/// 文字列表現から [`TemporalId`] を復元する。
///
/// `"seconds/min:max"`（単体なら`"seconds/index"`）形式の文字列をパースする。
#[cfg(feature = "temporal_id")]
impl FromStr for TemporalId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (i_text, t_text) = s.split_once('/').ok_or_else(|| parse_error(s))?;

        let seconds = i_text.parse::<u64>().map_err(|_| parse_error(s))?;
        let interval = Interval::new(seconds)?;

        let t = match t_text.split_once(':') {
            Some((a, b)) => [
                a.parse::<u64>().map_err(|_| parse_error(s))?,
                b.parse::<u64>().map_err(|_| parse_error(s))?,
            ],
            None => {
                let v = t_text.parse::<u64>().map_err(|_| parse_error(s))?;
                [v, v]
            }
        };

        TemporalId::new(interval, t)
    }
}

#[cfg(feature = "temporal_id")]
fn parse_error(input: &str) -> Error {
    SpatialIdError::ParseSpatialIdFormat {
        kind: "TemporalId",
        input: input.to_string(),
    }
    .into()
}

#[cfg(all(test, feature = "temporal_id"))]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    /// 交差結果がどちらかの被演算子と同じ区間なら、その[`Interval`]表記を保つ。
    #[test]
    fn intersection_keeps_operand_notation_when_range_is_unchanged() {
        let whole = TemporalId::WHOLE;
        let half_hour = TemporalId::new(1800, 809712).unwrap();

        assert_eq!(whole.intersection(&half_hour).unwrap(), half_hour);
        assert_eq!(half_hour.intersection(&whole).unwrap(), half_hour);
        assert_eq!(half_hour.intersection(&half_hour).unwrap(), half_hour);
    }

    /// `interval`が異なっていても絶対秒区間で正しく交差する。
    #[test]
    fn intersection_across_different_intervals() {
        // [3600, 7200) と [5400, 9000) の交差は [5400, 7200) = 30分ぶん。
        let hour = TemporalId::new(Interval::HOUR, 1).unwrap();
        let shifted = TemporalId::new(1800, [3, 4]).unwrap();

        let got = hour.intersection(&shifted).unwrap();
        assert_eq!(got.seconds_range(), (5400, 7200));

        // 重ならない場合。
        let far = TemporalId::new(Interval::HOUR, 100).unwrap();
        assert!(hour.intersection(&far).is_none());
    }

    /// 内側を刳り抜くと前後2つ、端を削ると1つ、覆われると0個になる。
    #[test]
    fn difference_splits_into_at_most_two_pieces() {
        let day = TemporalId::new(Interval::DAY, 0).unwrap(); // [0, 86400)
        let middle = TemporalId::new(Interval::HOUR, [1, 1]).unwrap(); // [3600, 7200)

        let pieces: Vec<_> = day.difference(&middle).collect();
        let ranges: Vec<_> = pieces.iter().map(|p| p.seconds_range()).collect();
        assert_eq!(ranges, [(0, 3600), (7200, 86400)]);

        // 端を削る。
        let head = TemporalId::new(Interval::HOUR, [0, 0]).unwrap();
        let pieces: Vec<_> = day.difference(&head).collect();
        assert_eq!(
            pieces.iter().map(|p| p.seconds_range()).collect::<Vec<_>>(),
            [(3600, 86400)]
        );

        // 完全に覆われる。
        assert_eq!(day.difference(&TemporalId::WHOLE).count(), 0);

        // 重ならない場合は self をそのまま返す（表記も保つ）。
        let far = TemporalId::new(Interval::DAY, 5).unwrap();
        let untouched: Vec<_> = day.difference(&far).collect();
        assert_eq!(untouched.as_slice(), core::slice::from_ref(&day));
    }

    /// 仕様書 1.5.3 の例（`i`=1800）のように2の冪でない間隔も、木のセルへ過不足なく分解される。
    #[test]
    fn segments_cover_exactly_the_same_seconds() {
        let id = TemporalId::new(1800, 809712).unwrap();
        let (start, end) = id.seconds_range();

        let cells: Vec<_> = id.segments().collect();
        assert!(cells.len() > 1, "2の冪でない間隔は複数セルへ分解される");

        // 生成順は区間木の都合で昇順にならないため、開始秒で並べ直してから連続性を見る。
        let mut ranges: Vec<_> = cells.iter().map(|c| c.seconds_range()).collect();
        ranges.sort_unstable();

        let mut cursor = start;
        for (s, e) in ranges {
            assert_eq!(s, cursor, "セルが隙間なく連続していない");
            cursor = e;
        }
        assert_eq!(cursor, end, "合計が元の区間と一致しない");
    }
}
