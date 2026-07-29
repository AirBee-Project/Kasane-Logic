#[cfg(feature = "temporal_id")]
use alloc::string::ToString;
#[cfg(feature = "temporal_id")]
use core::{fmt, str::FromStr};

#[cfg(feature = "temporal_id")]
use crate::{
    Interval, SpatialIdError, TemporalCell,
    error::Error,
    spatial_id::{helpers::format_dimension, temporal_zoom_level::TZoomLevel},
};

/// [`RangeId`](crate::RangeId)が保持する、人間に読みやすい時間区間の範囲表現。
///
/// `RangeId.f/x/y`が「単位（ズームレベル）＋範囲」であるのと同じ形で、時間の単位（[`Interval`]）と
/// その単位でのインデックス範囲 `[min, max]`（両端含む）を保持する。FlexTreeへ格納する際は
/// [`into_cells`](Self::into_cells)で生の2進セル（[`TemporalCell`]）の列へ分解される。
#[cfg(feature = "temporal_id")]
#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct TemporalRange {
    interval: Interval,
    t: [u64; 2],
}

#[cfg(feature = "temporal_id")]
impl TemporalRange {
    /// 全時間を表す定数。
    pub const WHOLE: Self = TemporalRange {
        interval: Interval::Whole,
        t: [0, 0],
    };

    /// このインスタンスが全時間を表す特別な値（[`WHOLE`](Self::WHOLE)）であるかを判定する。
    pub fn is_whole(&self) -> bool {
        self.interval == Interval::Whole && self.t == [0, 0]
    }

    /// 指定した単位・範囲から [`TemporalRange`] を構築する。
    ///
    /// `t` は自動的に昇順へ並び替えられる。範囲が表す絶対秒区間が
    /// [`Interval::WHOLE_SECONDS`] を超える場合は [`Error`] を返す。
    pub fn new(interval: Interval, t: [u64; 2]) -> Result<Self, Error> {
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

    /// 検証を行わずに [`TemporalRange`] を構築する。
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

    /// この [`TemporalRange`] を、絶対秒区間へ変換した上で、生の2進セル（[`TemporalCell`]）の列へ
    /// 分解する。
    ///
    /// [`interval()`](Self::interval)の秒数は2の冪であるとは限らない（Day/Hour/Minuteはいずれも
    /// 2の冪ではない）ため、区間木的な分解（`SegmentIter64`）により高々`O(log 秒数)`個の2進セルへ
    /// 分解される。
    pub fn into_cells(&self) -> impl Iterator<Item = TemporalCell> {
        let unit = self.interval.seconds();
        let start = self.t[0] * unit;
        let end_inclusive = (self.t[1] + 1) * unit - 1;

        split_t([start, end_inclusive])
            .map(|(zoom, index)| unsafe { TemporalCell::new_unchecked(zoom, index) })
    }
}

#[cfg(feature = "temporal_id")]
impl crate::spatial_id::traits::TemporalId for TemporalRange {
    const WHOLE: Self = Self::WHOLE;

    fn is_whole(&self) -> bool {
        Self::is_whole(self)
    }

    fn seconds_range(&self) -> (u64, u64) {
        let unit = self.interval.seconds();
        (self.t[0] * unit, (self.t[1] + 1) * unit)
    }
}

/// 生の2進セル（[`TemporalCell`]）1個を、被覆する絶対秒区間として最も粗く一致する
/// [`Interval`]ラベルで表した [`TemporalRange`] へ変換する。
///
/// [`Interval::coarse_to_fine`]の順に試し、`Second`は常に割り切れるため必ずどこかで成功する
/// （フォールバック分岐は不要）。
#[cfg(feature = "temporal_id")]
impl From<&TemporalCell> for TemporalRange {
    fn from(cell: &TemporalCell) -> Self {
        use crate::spatial_id::traits::TemporalId as _;

        let (start, end) = cell.seconds_range();
        let span = end - start;

        for interval in Interval::coarse_to_fine() {
            let unit = interval.seconds();
            if start % unit == 0 && span % unit == 0 {
                return TemporalRange {
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
impl fmt::Display for TemporalRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.interval, format_dimension(self.t))
    }
}

/// 文字列表現から [`TemporalRange`] を復元する。
///
/// `"seconds/min:max"`（単体なら`"seconds/index"`）形式の文字列をパースする。
#[cfg(feature = "temporal_id")]
impl FromStr for TemporalRange {
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

        TemporalRange::new(interval, t)
    }
}

#[cfg(feature = "temporal_id")]
fn parse_error(input: &str) -> Error {
    SpatialIdError::ParseSpatialIdFormat {
        kind: "TemporalRange",
        input: input.to_string(),
    }
    .into()
}
