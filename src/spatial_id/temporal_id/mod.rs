#[cfg(not(feature = "temporal_id"))]
mod disabled;
#[cfg(not(feature = "temporal_id"))]
pub use disabled::{TemporalId, TemporalMap, TemporalSet};

#[cfg(feature = "temporal_id")]
use crate::{SpatialIdError, error::Error};
#[cfg(feature = "temporal_id")]
use alloc::vec::Vec;
#[cfg(feature = "temporal_id")]
pub mod impls;
#[cfg(feature = "temporal_id")]
pub mod ops;
#[cfg(feature = "temporal_id")]
pub mod set;
#[cfg(feature = "temporal_id")]
pub use set::TemporalSet;
#[cfg(feature = "temporal_id")]
pub mod map;
#[cfg(feature = "temporal_id")]
pub use map::TemporalMap;
#[cfg(feature = "temporal_id")]
pub mod interval;
#[cfg(feature = "temporal_id")]
pub use interval::Interval;

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
    /// 全時間（ドメイン全体 `[0, DOMAIN_END)`）を表す時間ID。
    pub const WHOLE: TemporalId = TemporalId {
        interval: Interval::Whole,
        t: 0,
    };

    /// 時間ドメインの排他的終端（`86400 × 2^47` 秒）。
    pub const DOMAIN_END: u64 = interval::WHOLE_SECONDS;

    /// 時間間隔（[`Interval`]）と時間インデックス `t` から新しい [`TemporalId`] を作成する。
    ///
    /// # バリデーション
    /// 区間終端 `i * (t + 1)` が時間ドメイン終端 [`Self::DOMAIN_END`] を超える場合、
    /// [`Error::TOutOfRange`](crate::SpatialIdError::TOutOfRange) を返す。
    ///
    /// # 例
    ///
    /// ```
    /// # use kasane_logic::{Interval, TemporalId};
    /// let id = TemporalId::new(Interval::Hour, 5).unwrap();
    /// assert_eq!(id.interval(), Interval::Hour);
    /// assert_eq!(id.t(), 5);
    ///
    /// // 二進層（Day·2^k）も型安全に構築できる
    /// let two_days = TemporalId::new(Interval::day_pow(1).unwrap(), 0).unwrap();
    /// assert_eq!(two_days.i(), 86400 * 2);
    /// ```
    pub fn new(interval: Interval, t: u64) -> Result<Self, Error> {
        let i = interval.seconds();
        let end_exclusive = i as u128 * (t as u128 + 1);
        if end_exclusive > Self::DOMAIN_END as u128 {
            return Err(SpatialIdError::TOutOfRange { i, t }.into());
        }
        Ok(Self { interval, t })
    }

    /// 生の秒数 `i` と時間インデックス `t` から構築する（[`new`](Self::new) の秒数版）。
    ///
    /// 文字列パースや外部データ（Ouranos 仕様の `i/t` 表記）の取り込みに使う。
    /// プログラム内で間隔が静的に決まっている場合は [`new`](Self::new) を推奨する。
    ///
    /// # バリデーション
    ///
    /// - `i` が約数鎖に含まれない場合、
    ///   [`Error::TIntervalError`](crate::SpatialIdError::TIntervalError) を返す。
    /// - 区間終端 `i * (t + 1)` が時間ドメイン終端 [`Self::DOMAIN_END`] を超える場合、
    ///   [`Error::TOutOfRange`](crate::SpatialIdError::TOutOfRange) を返す。
    ///
    /// # 例
    ///
    /// 有効な時間IDの作成:
    /// ```
    /// # use kasane_logic::TemporalId;
    /// let id = TemporalId::from_seconds(3600, 5).unwrap();
    /// assert_eq!(id.i(), 3600);
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
    /// 戻り値は `i * (t + 1)` である。構築時に時間ドメイン `[0, DOMAIN_END)` へ
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

    /// 時間間隔 `i`（秒）を取得する。
    ///
    /// # 例
    ///
    /// ```
    /// # use kasane_logic::TemporalId;
    /// let id = TemporalId::from_seconds(3600, 5).unwrap();
    /// assert_eq!(id.i(), 3600);
    /// ```
    pub fn i(&self) -> u64 {
        self.interval.seconds()
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

    /// Ouranos 仕様の任意間隔 Temporal ID（`i/t`、`i` は任意の秒数）を、
    /// 等価な約数鎖セル列へ正規化する。
    ///
    /// 仕様（1.5.3）では時間間隔 `i` に任意の秒数を指定できるが、本ライブラリの
    /// ネイティブ表現は約数鎖（[`Interval`]）に限定される。この関数は
    /// 仕様準拠のIDが表す時間範囲 `[i*t, i*(t+1))` を [`from_range`](Self::from_range)
    /// で分解し、同じ範囲を過不足なく覆うセル列を返す。
    ///
    /// `i` が約数鎖に含まれる場合は単一のIDを返す。
    ///
    /// # バリデーション
    ///
    /// - `i == 0` の場合は [`SpatialIdError::TIntervalError`] を返す。
    /// - 範囲終端が時間ドメイン `[0, DOMAIN_END)` を超える場合は
    ///   [`SpatialIdError::TOutOfRange`] を返す。
    ///
    /// # 例
    ///
    /// 仕様書の例 `1800/809712`（30分間隔）の正規化:
    /// ```
    /// # use kasane_logic::TemporalId;
    /// let ids = TemporalId::from_spec(1800, 809712).unwrap();
    /// // [1457481600, 1457483400) = 30個の分セル
    /// assert_eq!(ids.len(), 30);
    /// assert!(ids.iter().all(|id| id.i() == 60));
    /// assert_eq!(ids[0].start_unixtime(), 1457481600);
    /// ```
    pub fn from_spec(i: u64, t: u64) -> Result<Vec<TemporalId>, Error> {
        if i == 0 {
            return Err(SpatialIdError::TIntervalError { i }.into());
        }
        if let Ok(interval) = Interval::new(i) {
            return Ok(alloc::vec![Self::new(interval, t)?]);
        }
        let start = i as u128 * t as u128;
        let end_exclusive = i as u128 * (t as u128 + 1);
        if end_exclusive > Self::DOMAIN_END as u128 {
            return Err(SpatialIdError::TOutOfRange { i, t }.into());
        }
        Self::from_range(start as u64, end_exclusive as u64)
    }

    /// 開始と終了（排他的）のUNIXタイムスタンプから、時間範囲を表す最小個数の [`TemporalId`] 列を生成する。
    ///
    /// 与えられた時間範囲 `[start, end_exclusive)` を表現するために、
    /// 最も大きな時間間隔から貪欲に [`TemporalId`] を切り出す。
    /// 約数鎖に二進層（`Day·2^k`）があるため、生成されるセル数はドメイン内の
    /// どんな範囲でも高々数百個に収まる。
    ///
    /// # パラメーター
    ///
    /// * `start` — 時間範囲の開始（UNIXタイムスタンプ、秒単位）
    /// * `end_exclusive` — 時間範囲の終了（UNIXタイムスタンプ、秒単位、排他的）
    ///
    /// # バリデーション
    ///
    /// - `start >= end_exclusive` の場合、空のベクトルを返す。
    /// - `end_exclusive` が [`Self::DOMAIN_END`] を超える場合、
    ///   [`SpatialIdError::TOutOfRange`] を返す。
    ///
    /// # 例
    ///
    /// 1時間の範囲:
    /// ```
    /// # use kasane_logic::TemporalId;
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
    /// # use kasane_logic::TemporalId;
    /// let ids = TemporalId::from_range(0, TemporalId::DOMAIN_END).unwrap();
    /// assert_eq!(ids, vec![TemporalId::WHOLE]);
    /// ```
    pub fn from_range(start: u64, end_exclusive: u64) -> Result<Vec<TemporalId>, Error> {
        if start >= end_exclusive {
            return Ok(alloc::vec![]);
        }
        if end_exclusive > Self::DOMAIN_END {
            return Err(SpatialIdError::TOutOfRange {
                i: 1,
                t: end_exclusive - 1,
            }
            .into());
        }
        Ok(Self::decompose(start, end_exclusive))
    }

    /// `[start, end_exclusive)`（非空・ドメイン内であること）を約数鎖セル列へ貪欲分解する。
    ///
    /// 二進層があるため出力は高々数百セル。
    pub(crate) fn decompose(start: u64, end_exclusive: u64) -> Vec<TemporalId> {
        debug_assert!(start < end_exclusive && end_exclusive <= Self::DOMAIN_END);

        let mut result = Vec::new();
        let mut current = start;

        while current < end_exclusive {
            let remaining = end_exclusive - current;

            for interval in Interval::coarse_to_fine() {
                let secs = interval.seconds();
                if current.is_multiple_of(secs) && remaining >= secs {
                    // ドメイン内（current + secs <= end_exclusive <= DOMAIN_END）なので直接構築できる。
                    result.push(TemporalId {
                        interval,
                        t: current / secs,
                    });
                    current += secs;
                    break;
                }
                // Interval::SECOND（1秒）は必ず条件を満たすため、このループは必ず1セル進む。
            }
        }

        result
    }
}
