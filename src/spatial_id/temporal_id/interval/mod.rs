// temporal_id feature 無効時は専用のスタブを使う。
#[cfg(not(feature = "temporal_id"))]
mod disabled;
#[cfg(not(feature = "temporal_id"))]
pub use disabled::Interval;

#[cfg(feature = "temporal_id")]
use crate::Error;

#[cfg(feature = "temporal_id")]
mod impls;

/// 時間IDの時間間隔`i`を表現する型。`i`が任意の0より大きい自然数を受け入れると、処理に不整合が生じるためパターンを限定する。
///
/// | バリアント | 秒数 |
/// |---|---|
/// | [`Whole`](Self::Whole) | `2^62`（約1,460億年） |
/// | [`Day`](Self::Day) | 86400 |
/// | [`Hour`](Self::Hour) | 3600 |
/// | [`Minute`](Self::Minute) | 60 |
/// | [`Second`](Self::Second) | 1 |
#[cfg(feature = "temporal_id")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum Interval {
    /// 全時間（`2^62`秒）
    Whole,
    /// 1日（86400 秒）
    Day,
    /// 1時間（3600 秒）
    Hour,
    /// 1分（60 秒）
    Minute,
    /// 1秒（1 秒）
    Second,
}

#[cfg(feature = "temporal_id")]
impl Interval {
    /// このライブラリが扱える全時間の秒数。
    ///
    /// [`TZoomLevel::MAX`](crate::spatial_id::temporal_id::zoom_level::TZoomLevel::MAX)（生の2進セル表現が
    /// 表現できる最深ズーム）に対応する `2^WHOLE_POW` と厳密に一致していなければならない。ここがずれると、
    /// `TemporalSegment::WHOLE`（生セル）と`Interval::Whole`（人間向けラベル）とで「全時間」が指す絶対秒区間が
    /// 食い違い、`TemporalRange`⇄`TemporalSegment`間の変換で`Whole`が正しく認識されなくなる。
    pub const WHOLE_SECONDS: u64 = 1u64 << Self::WHOLE_POW;

    /// 最も粗い時間区間を表す二進層の指数。`TZoomLevel::MAX`と一致させること。
    pub const WHOLE_POW: u8 = 62;

    /// 秒数から[Interval]型を作成する。
    ///
    /// [Interval]に当てはまらない場合は [`SpatialIdError::TIntervalError`](crate::SpatialIdError::TIntervalError) を返す。
    ///
    /// # 例
    ///
    /// ```
    /// # use kasane_logic::Interval;
    /// assert_eq!(Interval::new(3600).unwrap(), Interval::Hour);
    /// assert!(Interval::new(7200).is_err()); // 候補に無い
    /// ```
    pub fn new(seconds: u64) -> Result<Interval, Error> {
        Interval::try_from(seconds)
    }

    /// この[Interval]の秒数。
    pub const fn seconds(self) -> u64 {
        match self {
            Interval::Whole => Self::WHOLE_SECONDS,
            Interval::Day => 86400,
            Interval::Hour => 3600,
            Interval::Minute => 60,
            Interval::Second => 1,
        }
    }

    /// [Interval]の候補を粗い→細かい順に列挙する。
    pub fn coarse_to_fine() -> impl Iterator<Item = Interval> {
        core::iter::once(Interval::Whole).chain([
            Interval::Day,
            Interval::Hour,
            Interval::Minute,
            Interval::Second,
        ])
    }
}
