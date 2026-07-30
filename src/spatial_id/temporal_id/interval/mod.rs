// temporal_id feature 無効時は専用のスタブを使う。
#[cfg(not(feature = "temporal_id"))]
mod disabled;
#[cfg(not(feature = "temporal_id"))]
pub use disabled::Interval;

#[cfg(feature = "temporal_id")]
use crate::{Error, SpatialIdError};

#[cfg(feature = "temporal_id")]
mod impls;

/// 時間IDの時間間隔`i`（秒数）を表現する型。
///
/// [Ouranos 4D Spatio-temporal ID仕様](https://github.com/AirBee-Project)の`Temporal ID`は
/// 「任意の秒数」を時間間隔として許容するため、`1..=`[`WHOLE_SECONDS`](Self::WHOLE_SECONDS)の
/// 範囲であれば任意の値を保持できる（`Day`/`Hour`のような固定候補への限定はしない）。
///
/// よく使う値は関連定数として用意している。
///
/// | 定数 | 秒数 |
/// |---|---|
/// | [`WHOLE`](Self::WHOLE) | `2^62`（約1,460億年） |
/// | [`DAY`](Self::DAY) | 86400 |
/// | [`HOUR`](Self::HOUR) | 3600 |
/// | [`MINUTE`](Self::MINUTE) | 60 |
/// | [`SECOND`](Self::SECOND) | 1 |
#[cfg(feature = "temporal_id")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct Interval(u64);

#[cfg(feature = "temporal_id")]
impl Interval {
    /// このライブラリが扱える全時間の秒数。
    pub const WHOLE_SECONDS: u64 = 1u64 << Self::WHOLE_POW;

    /// 最も粗い時間区間を表す二進層の指数。`TZoomLevel::MAX`と一致させること。
    pub const WHOLE_POW: u8 = 62;

    /// 全時間（`2^62`秒）。
    pub const WHOLE: Interval = Interval(Self::WHOLE_SECONDS);
    /// 1日（86400秒）。
    pub const DAY: Interval = Interval(86400);
    /// 1時間（3600秒）。
    pub const HOUR: Interval = Interval(3600);
    /// 1分（60秒）。
    pub const MINUTE: Interval = Interval(60);
    /// 1秒（1秒）。
    pub const SECOND: Interval = Interval(1);

    /// 秒数から[Interval]型を作成する。
    ///
    /// `seconds`が`0`、または[`WHOLE_SECONDS`](Self::WHOLE_SECONDS)を超える場合は
    /// [`SpatialIdError::TIntervalError`] を返す。
    ///
    /// # 例
    ///
    /// ```
    /// # use kasane_logic::Interval;
    /// assert_eq!(Interval::new(3600).unwrap(), Interval::HOUR);
    /// assert_eq!(Interval::new(7200).unwrap().seconds(), 7200); // 候補以外の秒数も許容する
    /// assert!(Interval::new(0).is_err());
    /// ```
    pub fn new(seconds: u64) -> Result<Interval, Error> {
        if seconds == 0 || seconds > Self::WHOLE_SECONDS {
            return Err(SpatialIdError::TIntervalError { i: seconds }.into());
        }
        Ok(Interval(seconds))
    }

    /// この[Interval]の秒数。
    pub const fn seconds(self) -> u64 {
        self.0
    }

    /// 秒数がきれいな候補（[`WHOLE`](Self::WHOLE)/[`DAY`](Self::DAY)/[`HOUR`](Self::HOUR)/
    /// [`MINUTE`](Self::MINUTE)/[`SECOND`](Self::SECOND)）を粗い→細かい順に列挙する。
    ///
    /// 生の2進セルを人間に読みやすい[`TemporalId`](crate::TemporalId)へ復元する際、
    /// この順に一致を試すことで「読出し時になるべく大きい/きれいな単位で返す」ことができる
    /// （[`SECOND`](Self::SECOND)は常に割り切れるため、この列挙は必ずどこかで一致する）。
    pub fn coarse_to_fine() -> impl Iterator<Item = Interval> {
        [
            Self::WHOLE,
            Self::DAY,
            Self::HOUR,
            Self::MINUTE,
            Self::SECOND,
        ]
        .into_iter()
    }
}
