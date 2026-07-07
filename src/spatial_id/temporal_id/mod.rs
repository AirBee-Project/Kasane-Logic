#[cfg(not(feature = "temporal_id"))]
mod disabled;
#[cfg(not(feature = "temporal_id"))]
pub use disabled::{Interval, TemporalId, TemporalMap, TemporalSet};

#[cfg(feature = "temporal_id")]
use crate::{SpatialIdError, error::Error};
#[cfg(feature = "temporal_id")]
use alloc::vec::Vec;
#[cfg(feature = "temporal_id")]
pub mod collection;
#[cfg(feature = "temporal_id")]
pub mod impls;
#[cfg(feature = "temporal_id")]
pub mod ops;
#[cfg(feature = "temporal_id")]
pub use collection::map::TemporalMap;
#[cfg(feature = "temporal_id")]
pub use collection::set::TemporalSet;
#[cfg(feature = "temporal_id")]
pub mod interval;
#[cfg(feature = "temporal_id")]
pub use interval::Interval;
#[cfg(all(test, feature = "temporal_id"))]
mod tests;

#[cfg(feature = "temporal_id")]
#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
/// 時間IDの区間表現を表す型である。
///
/// [`TemporalId`] は、時間間隔 [`Interval`] と時間インデックス `t` の組み合わせで、
/// 時間範囲 `[i*t, i*(t+1))` を表現する。
/// 1時間単位のIDの作成:
/// ```
/// # use kasane_logic::TemporalId;
/// let id = TemporalId::from_seconds(3600, 10).unwrap();
/// assert_eq!(id.start_unixtime(), 36000);
/// assert_eq!(id.end_unixtime_exclusive(), 39600);
/// ```
pub struct TemporalId {
    interval: Interval,
    t: u64,
}

#[cfg(feature = "temporal_id")]
impl TemporalId {
    /// 全時間（ドメイン全体 `[0, WHOLE_SECONDS)`）を表す時間ID。
    pub const WHOLE: TemporalId = TemporalId {
        interval: Interval::Whole,
        t: 0,
    };

    /// 時間間隔（[`Interval`]）と時間インデックス `t` から新しい [`TemporalId`] を作成する。
    ///
    /// # 例
    ///
    /// ```
    /// # use kasane_logic::{Interval, TemporalId};
    /// let id = TemporalId::new(Interval::Hour, 5).unwrap();
    /// assert_eq!(id.interval(), Interval::Hour);
    /// assert_eq!(id.t(), 5);
    ///
    /// // Day·2^kも安全に構築できる
    /// let two_days = TemporalId::new(Interval::day_pow(1).unwrap(), 0).unwrap();
    /// assert_eq!(two_days.i().seconds(), 86400 * 2);
    /// ```
    pub fn new<I>(interval: I, t: u64) -> Result<Self, Error>
    where
        I: TryInto<Interval>,
        Error: From<I::Error>,
    {
        let interval = interval.try_into()?;
        let i = interval.seconds();
        let end_exclusive = i as u128 * (t as u128 + 1);
        if end_exclusive > Interval::WHOLE_SECONDS as u128 {
            return Err(SpatialIdError::TOutOfRange { i, t }.into());
        }
        Ok(Self { interval, t })
    }

    /// 生の秒数 `i` と時間インデックス `t` から構築する（[`new`](Self::new) の秒数版）。
    ///
    /// 文字列パースや外部データ（Ouranos 仕様 of `i/t` 表記）の取り込みに使う。
    /// プログラム内で間隔が静的に決まっている場合は [`new`](Self::new) を推奨する。
    ///
    /// # バリデーション
    ///
    /// - `i` が約数鎖に含まれない場合、
    ///   [`Error::TIntervalError`](crate::SpatialIdError::TIntervalError) を返す。
    /// - 区間終端 `i * (t + 1)` が時間ドメイン終端 [`Interval::WHOLE_SECONDS`] を超える場合、
    ///   [`Error::TOutOfRange`](crate::SpatialIdError::TOutOfRange) を返す。
    ///
    /// # 例
    ///
    /// 有効な時間IDの作成:
    /// ```
    /// # use kasane_logic::{TemporalId, Interval};
    /// let id = TemporalId::from_seconds(3600, 5).unwrap();
    /// assert_eq!(id.i(), Interval::Hour);
    /// assert_eq!(id.t(), 5);
    /// ```
    ///
    /// 無効な時間間隔の検知:
    /// ```
    /// # use kasane_logic::TemporalId;
    /// let id = TemporalId::from_seconds(7200, 5);
    /// assert!(id.is_err());
    /// ```
    pub fn from_seconds(i: u64, t: u64) -> Result<Self, Error> {
        let interval = Interval::new(i)?;
        Self::new(interval, t)
    }

    /// この時間IDの間隔（[`Interval`] 型）。
    pub fn interval(&self) -> Interval {
        self.interval
    }

    /// このインスタンスが全時間を表す特別な値（`WHOLE`）であるかを判定する。
    ///
    /// # 例
    ///
    /// ```
    /// # use kasane_logic::TemporalId;
    /// let whole = TemporalId::WHOLE;
    /// assert!(whole.is_whole());
    ///
    /// let specific = TemporalId::from_seconds(3600, 5).unwrap();
    /// assert!(!specific.is_whole());
    /// ```
    pub fn is_whole(&self) -> bool {
        self.interval == Interval::Whole
    }

    /// この時間区間の開始時刻をUNIXタイムスタンプ（秒単位）で取得する。
    ///
    /// 戻り値は `i * t` である。
    ///
    /// # 例
    ///
    /// ```
    /// # use kasane_logic::TemporalId;
    /// let id = TemporalId::from_seconds(3600, 10).unwrap();
    /// assert_eq!(id.start_unixtime(), 36000);
    /// ```
    pub fn start_unixtime(&self) -> u64 {
        self.interval.seconds() * self.t
    }

    /// この時間区間の終了時刻をUNIXタイムスタンプ（秒単位、排他的）で取得する。
    ///
    /// 戻り値は `i * (t + 1)` である。構築時に時間ドメイン `[0, WHOLE_SECONDS)` へ
    /// 収まることが検証されているため、`u64` に必ず収まる。
    ///
    /// # 例
    ///
    /// ```
    /// # use kasane_logic::TemporalId;
    /// let id = TemporalId::from_seconds(3600, 10).unwrap();
    /// assert_eq!(id.end_unixtime_exclusive(), 39600);
    /// ```
    pub fn end_unixtime_exclusive(&self) -> u64 {
        self.interval.seconds() * (self.t + 1)
    }

    /// 時間間隔を取得する。
    ///
    /// # 例
    ///
    /// ```
    /// # use kasane_logic::{TemporalId, Interval};
    /// let id = TemporalId::from_seconds(3600, 5).unwrap();
    /// assert_eq!(id.i(), Interval::Hour);
    /// ```
    pub fn i(&self) -> Interval {
        self.interval
    }

    /// 時間インデックス `t` を取得する。
    ///
    /// # 例
    ///
    /// ```
    /// # use kasane_logic::TemporalId;
    /// let id = TemporalId::from_seconds(3600, 5).unwrap();
    /// assert_eq!(id.t(), 5);
    /// ```
    pub fn t(&self) -> u64 {
        self.t
    }

    /// 開始と終了（排他的）のUNIXタイムスタンプから、時間範囲を表す最小個数の [`TemporalId`] 列を生成する。
    ///
    /// # パラメーター
    ///
    /// * `start` — 時間範囲の開始（UNIXタイムスタンプ、秒単位）
    /// * `end_exclusive` — 時間範囲の終了（UNIXタイムスタンプ、秒単位、排他的）
    ///
    /// # バリデーション
    ///
    /// - `start >= end_exclusive` の場合、空のベクトルを返す。
    /// - `end_exclusive` が [`Interval::WHOLE_SECONDS`] を超える場合、
    ///   [`SpatialIdError::TOutOfRange`] を返す。
    ///
    /// # 例
    ///
    /// 1時間の範囲:
    /// ```
    /// # use kasane_logic::{TemporalId, Interval};
    /// let ids = TemporalId::from_range(0, 3600).unwrap();
    /// assert_eq!(ids.len(), 1);
    /// assert_eq!(ids[0], TemporalId::from_seconds(3600, 0).unwrap());
    /// ```
    ///
    /// 複雑な範囲（時間と分の組み合わせ）:
    /// ```
    /// # use kasane_logic::TemporalId;
    /// let ids = TemporalId::from_range(0, 3720).unwrap(); // 1時間 + 2分
    /// assert!(ids.len() >= 1);
    /// // 最初の要素は3600秒（1時間）の間隔を持つ
    /// ```
    ///
    /// ドメイン全体は単一の WHOLE セル:
    /// ```
    /// # use kasane_logic::{TemporalId, Interval};
    /// let ids = TemporalId::from_range(0, Interval::WHOLE_SECONDS).unwrap();
    /// assert_eq!(ids, vec![TemporalId::WHOLE]);
    /// ```
    pub fn from_range(start: u64, end_exclusive: u64) -> Result<Vec<TemporalId>, Error> {
        if start >= end_exclusive {
            return Ok(alloc::vec![]);
        }
        if end_exclusive > Interval::WHOLE_SECONDS {
            return Err(SpatialIdError::TOutOfRange {
                i: 1,
                t: end_exclusive - 1,
            }
            .into());
        }

        Ok(Self::cells_in_range(start, end_exclusive).collect())
    }

    /// `[start, end_exclusive)` を約数鎖の最小セル列へ**遅延**分解するイテレータを返す。
    ///
    /// [`from_range`](Self::from_range) の中間 `Vec` を省く軽量版で、走査系のホットパス
    /// （列挙・切り取り・個数計上）で使う。呼び出し側は `end_exclusive <=`
    /// [`Interval::WHOLE_SECONDS`] を保証すること（[`TemporalCore`](crate::spatial_id::temporal_id::collection::temporal_core::TemporalCore)
    /// のセグメントは常に有効域内なので満たされる）。`start >= end_exclusive` は空を返す。
    pub(crate) fn cells_in_range(start: u64, end_exclusive: u64) -> RangeCells {
        RangeCells {
            current: start,
            end: end_exclusive,
        }
    }

    /// `[start, end_exclusive)` が分解されるセル個数を、セルを生成せずに数える。
    ///
    /// 二進層（`Day·2^k`）のおかげで個数は対数オーダーに収まるため、割当なしで軽い。
    pub(crate) fn count_range(start: u64, end_exclusive: u64) -> usize {
        Self::cells_in_range(start, end_exclusive).count()
    }
}

/// [`TemporalId::cells_in_range`] の遅延イテレータ。各 `next` で約数鎖の最coarse に
/// 整合するセルを1つ返し、カーソルを進める（割当なし）。
#[cfg(feature = "temporal_id")]
pub(crate) struct RangeCells {
    current: u64,
    end: u64,
}

#[cfg(feature = "temporal_id")]
impl Iterator for RangeCells {
    type Item = TemporalId;

    #[allow(clippy::manual_is_multiple_of)]
    fn next(&mut self) -> Option<TemporalId> {
        if self.current >= self.end {
            return None;
        }
        let remaining = self.end - self.current;
        for interval in Interval::coarse_to_fine() {
            let secs = interval.seconds();
            if self.current % secs == 0 && remaining >= secs {
                let cell = TemporalId {
                    interval,
                    t: self.current / secs,
                };
                self.current += secs;
                return Some(cell);
            }
        }
        // Second（secs=1）が必ず整合するため到達しない。
        None
    }
}
