#[cfg(feature = "temporal_id")]
use alloc::string::ToString;
#[cfg(feature = "temporal_id")]
use core::{fmt, str::FromStr};

#[cfg(feature = "temporal_id")]
use crate::{
    Interval, SpatialIdError, TemporalSegment,
    error::Error,
    spatial_id::{helpers::format_dimension, temporal_id::zoom_level::TZoomLevel},
};

/// 時間区間を表す、人間に読みやすい公開表現。
///
/// [`FlexId`](crate::FlexId)/[`SingleId`](crate::SingleId)/[`RangeId`](crate::RangeId)の
/// いずれも [`SpatialId::temporal`](crate::SpatialId::temporal)/
/// [`SpatialId::try_with_temporal`](crate::SpatialId::try_with_temporal)を通じてこの1つの型だけを
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
        interval: Interval::Whole,
        t: [0, 0],
    };

    /// このインスタンスが全時間を表す特別な値（[`WHOLE`](Self::WHOLE)）であるかを判定する。
    pub fn is_whole(&self) -> bool {
        self.interval == Interval::Whole && self.t == [0, 0]
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
    /// `t` は自動的に昇順へ並び替えられる。範囲が表す絶対秒区間が
    /// [`Interval::WHOLE_SECONDS`] を超える場合は [`Error`] を返す。
    pub fn new(interval: impl Into<i64>, t: [u64; 2]) -> Result<Self, Error> {
        let seconds: i64 = interval.into();
        let seconds =
            u64::try_from(seconds).map_err(|_| SpatialIdError::TIntervalError { i: 0 })?;
        let interval = Interval::new(seconds)?;
        let mut t = t;
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
///
/// [`Interval::coarse_to_fine`]の順に試し、`Second`は常に割り切れるため必ずどこかで成功する
/// （フォールバック分岐は不要）。
#[cfg(feature = "temporal_id")]
impl From<&TemporalSegment> for TemporalId {
    fn from(cell: &TemporalSegment) -> Self {
        let (start, end) = cell.seconds_range();
        let span = end - start;

        for interval in Interval::coarse_to_fine() {
            let unit = interval.seconds();
            if start % unit == 0 && span % unit == 0 {
                return TemporalId {
                    interval,
                    t: [start / unit, end / unit - 1],
                };
            }
        }

        unreachable!("Interval::Second (1秒) は常に割り切れるため、ここには到達しない")
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
