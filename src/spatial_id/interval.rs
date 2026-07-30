//! 時間間隔 `{i}`。**公開 API に出てくる唯一の時間の型**である。
//!
//! 「いつか」を表す `{t}` は空間 ID 側（[`SingleId`](crate::SingleId) /
//! [`RangeId`](crate::RangeId) / [`FlexId`](crate::FlexId)）がフィールドとして直接持ち、
//! `interval()` / `t()` / `seconds_range()` で読み書きする。

use crate::{SpatialIdError, error::Error};

/// 時間 ID の時間間隔 `{i}`（秒数）を表現する型。
///
/// [Ouranos 4D 時空間ID仕様](https://github.com/AirBee-Project)の Temporal ID は
/// 「任意の秒数」を時間間隔として許容するため、`1..=`[`WHOLE_SECONDS`](Self::WHOLE_SECONDS)
/// の範囲であれば任意の値を保持できる（`Day`/`Hour` のような固定候補への限定はしない）。
///
/// よく使う値は関連定数として用意している。
///
/// | 定数 | 秒数 |
/// |---|---|
/// | [`WHOLE`](Self::WHOLE) | `2^35`（約1089年） |
/// | [`DAY`](Self::DAY) | 86400 |
/// | [`HOUR`](Self::HOUR) | 3600 |
/// | [`MINUTE`](Self::MINUTE) | 60 |
/// | [`SECOND`](Self::SECOND) | 1 |
///
/// # `temporal_id` feature 無効時
///
/// 中身の無いスタブ（サイズ0）になり、[`WHOLE`](Self::WHOLE) 以外は構築できない。
/// 各 ID が持つ `interval` フィールドは1バイトも消費しない。
#[cfg(feature = "temporal_id")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[repr(transparent)]
pub struct Interval(u64);

/// `temporal_id` feature 無効時の [`Interval`]。常に全時間を表すサイズ0のスタブ。
#[cfg(not(feature = "temporal_id"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct Interval;

impl Interval {
    /// このライブラリが扱える全時間の秒数（`2^35` 秒 = 34,359,738,368 秒、約1089年）。
    ///
    /// 起点は仕様どおり 1970-01-01 00:00 UTC なので、**3058年ごろまで**を表せる。
    pub const WHOLE_SECONDS: u64 = 1u64 << Self::WHOLE_POW;

    /// 最も粗い時間区間を表す二進層の指数。時間軸の最大ズームレベルでもある
    /// （[`TZoomLevel`](crate::spatial_id::zoom_level::TZoomLevel) の `LIMIT`。
    /// 両者が食い違わないことは `zoom_level.rs` の `const` アサーションが保証する）。
    ///
    /// # なぜ 35 なのか
    ///
    /// 時間軸は Unix 元期を起点とする2進トライで、最深ズームの1セルが1秒である。
    /// したがって `WHOLE_POW` はそのままトライの深さになり、現実の時刻がどれだけ深い
    /// ところに沈むかを決める。
    ///
    /// | `WHOLE_POW` | 表せる範囲 | 深さ |
    /// |---|---|---|
    /// | 30（空間軸と同じ） | 2004年まで | 30 |
    /// | 31 | 2038年まで | 31 |
    /// | 32 | 2106年まで | 32 |
    /// | **35** | **3058年まで** | **35** |
    /// | 62 | 1461億年後まで | 62 |
    ///
    /// 空間軸の最大ズーム（30）に合わせると2004年で尽きるため揃えられない。実データの
    /// 射程（気候予測などは2100年以降も扱う）を考えると32も狭い。35なら1000年以上の
    /// 余裕があり、深さは62の場合の56%で済む。
    pub const WHOLE_POW: u8 = 35;

    /// 全時間（`2^35` 秒）。時間を指定していない ID はこの値を持つ。
    #[cfg(feature = "temporal_id")]
    pub const WHOLE: Interval = Interval(Self::WHOLE_SECONDS);
    /// 全時間。`temporal_id` feature 無効時に唯一有効な値。
    #[cfg(not(feature = "temporal_id"))]
    pub const WHOLE: Interval = Interval;

    /// 1日（86400秒）。
    #[cfg(feature = "temporal_id")]
    pub const DAY: Interval = Interval(86_400);
    /// 1時間（3600秒）。
    #[cfg(feature = "temporal_id")]
    pub const HOUR: Interval = Interval(3_600);
    /// 1分（60秒）。
    #[cfg(feature = "temporal_id")]
    pub const MINUTE: Interval = Interval(60);
    /// 1秒。
    #[cfg(feature = "temporal_id")]
    pub const SECOND: Interval = Interval(1);

    /// 秒数から [`Interval`] を作成する。
    ///
    /// # バリデーション
    /// - `seconds` が `0`、または [`WHOLE_SECONDS`](Self::WHOLE_SECONDS) を超える場合は
    ///   [`SpatialIdError::TIntervalError`] を返す。
    /// - `temporal_id` feature 無効時は [`WHOLE_SECONDS`](Self::WHOLE_SECONDS) 以外を拒否する。
    ///
    /// ```
    /// # use kasane_logic::Interval;
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// assert_eq!(Interval::new(3600).unwrap(), Interval::HOUR);
    /// // 仕様どおり、候補以外の秒数も許容する。
    /// assert_eq!(Interval::new(1800).unwrap().seconds(), 1800);
    /// assert!(Interval::new(0).is_err());
    /// # }
    /// ```
    pub const fn new(seconds: u64) -> Result<Interval, Error> {
        #[cfg(feature = "temporal_id")]
        {
            if seconds == 0 || seconds > Self::WHOLE_SECONDS {
                return Err(Error::SpatialId(SpatialIdError::TIntervalError {
                    i: seconds,
                }));
            }
            Ok(Interval(seconds))
        }

        #[cfg(not(feature = "temporal_id"))]
        {
            if seconds == Self::WHOLE_SECONDS {
                Ok(Interval)
            } else {
                Err(Error::SpatialId(SpatialIdError::TIntervalError {
                    i: seconds,
                }))
            }
        }
    }

    /// この [`Interval`] の秒数。
    pub const fn seconds(self) -> u64 {
        #[cfg(feature = "temporal_id")]
        {
            self.0
        }

        #[cfg(not(feature = "temporal_id"))]
        {
            Self::WHOLE_SECONDS
        }
    }

    /// この単位で、Unix 時刻（秒）が属するセルのインデックス `{t}` を返す。
    ///
    /// 仕様書 1.5.3 (3) の `t = floor(u / i)`。
    ///
    /// ```
    /// # use kasane_logic::Interval;
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// assert_eq!(Interval::HOUR.index_of(7_300), 2);
    /// assert_eq!(Interval::new(1800).unwrap().index_of(1_457_482_000), 809_712);
    /// # }
    /// ```
    pub const fn index_of(self, unix_seconds: u64) -> u64 {
        unix_seconds / self.seconds()
    }

    /// 検証済みの秒数から直接構築する。クレート内部専用。
    ///
    /// 呼び出し側は `1 <= seconds <= `[`WHOLE_SECONDS`](Self::WHOLE_SECONDS) を保証すること。
    /// 絶対秒区間からの復元（[`coarsest_unit`](super::time_cells::coarsest_unit) の結果）の
    /// ように、構成上その範囲に収まることが証明できる経路でのみ使う。
    pub(crate) const fn from_seconds_unchecked(seconds: u64) -> Interval {
        #[cfg(feature = "temporal_id")]
        {
            Interval(seconds)
        }

        #[cfg(not(feature = "temporal_id"))]
        {
            debug_assert!(seconds == Self::WHOLE_SECONDS);
            Interval
        }
    }

    /// 符号付きの秒数から構築する。負の値は [`SpatialIdError::TIntervalError`] になる。
    ///
    /// 無注釈の整数リテラルの既定型が `i32` であることに合わせ、`impl Into<i64>` を受ける
    /// API（`with_time` など）から使うための入口。
    pub(crate) fn from_signed_seconds(seconds: i64) -> Result<Interval, Error> {
        let seconds =
            u64::try_from(seconds).map_err(|_| SpatialIdError::TIntervalError { i: 0 })?;
        Self::new(seconds)
    }
}

impl core::fmt::Display for Interval {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.seconds())
    }
}

impl From<Interval> for i64 {
    fn from(interval: Interval) -> i64 {
        interval.seconds() as i64
    }
}

impl TryFrom<u64> for Interval {
    type Error = Error;

    fn try_from(seconds: u64) -> Result<Self, Self::Error> {
        Interval::new(seconds)
    }
}

macro_rules! impl_try_from_integer {
    ($($t:ty),*) => {
        $(
            impl TryFrom<$t> for Interval {
                type Error = Error;

                fn try_from(seconds: $t) -> Result<Self, Self::Error> {
                    Self::try_from(seconds as u64)
                }
            }
        )*
    };
}

impl_try_from_integer!(u8, u16, u32, u128, usize, i8, i16, i32, i128, isize);

#[cfg(all(test, feature = "temporal_id"))]
mod tests {
    use super::*;

    #[test]
    fn rejects_out_of_range_seconds() {
        assert!(Interval::new(0).is_err());
        assert!(Interval::new(Interval::WHOLE_SECONDS + 1).is_err());
        assert!(Interval::new(Interval::WHOLE_SECONDS).is_ok());
    }

    #[test]
    fn index_of_follows_the_spec_formula() {
        // t = floor(u / i)
        for (i, u, expected) in [
            (3600u64, 7_300u64, 2u64),
            (1800, 1_457_482_000, 809_712),
            (1, 1_770_000_000, 1_770_000_000),
            (86_400, 86_399, 0),
        ] {
            assert_eq!(Interval::new(i).unwrap().index_of(u), expected);
        }
    }
}
