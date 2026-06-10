#[cfg(not(feature = "temporal_id"))]
mod disabled;
#[cfg(not(feature = "temporal_id"))]
pub use disabled::TemporalId;

#[cfg(feature = "temporal_id")]
use crate::{SpatialIdError, error::Error};
#[cfg(feature = "temporal_id")]
use alloc::vec::Vec;
#[cfg(feature = "temporal_id")]
pub mod impls;
#[cfg(feature = "temporal_id")]
pub mod ops;

#[cfg(feature = "temporal_id")]
#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
/// 時間IDの区間表現を表す型である。
///
/// [`TemporalId`] は、時間間隔 `i` と時間インデックス `t` の組み合わせで、
/// 時間範囲 `[i*t, i*(t+1))` をUNIXタイムスタンプで表現する。
/// これにより、異なるスケールの時間区間を統一的に扱うことができる。
///
/// # 時間間隔について
///
/// `i` の有効な値は以下の定数で定義されている：
/// - `u64::MAX` — 全時間を表す特別な値
/// - `86400` — 1日（秒単位）
/// - `3600` — 1時間（秒単位）
/// - `60` — 1分（秒単位）
/// - `1` — 1秒（秒単位）
///
/// # 例
///
/// 1時間単位のIDの作成:
/// ```
/// # #[cfg(feature = "temporal_id")]
/// # {
/// # use kasane_logic::TemporalId;
/// let id = TemporalId::new(3600, 10).unwrap();
/// assert_eq!(id.start_unixstamp(), 36000);
/// assert_eq!(id.end_unixstamp_inclusive(), 39599);
/// # }
/// ```
pub struct TemporalId {
    /// 時間間隔（秒単位）。
    /// [`Self::TEMPORAL_I`] に定義された値のいずれかである。
    i: u64,
    /// 時間インデックス。
    /// この値と時間間隔 `i` を組み合わせることで、
    /// 実際の時間範囲 `[i*t, i*(t+1))` が決定される。
    t: u64,
}

#[cfg(feature = "temporal_id")]
impl TemporalId {
    pub const WHOLE: TemporalId = TemporalId { i: u64::MAX, t: 0 };
    pub const TEMPORAL_I: [u64; 5] = [u64::MAX, 86400, 3600, 60, 1];

    /// 指定された時間間隔と時間インデックスから新しい [`TemporalId`] を構築する。
    ///
    /// 与えられた `i` と `t` が有効な値であるかを検証し、
    /// 検証に失敗した場合は [`Error`] を返す。
    ///
    /// # パラメーター
    ///
    /// * `i` — 時間間隔（秒単位）。[`Self::TEMPORAL_I`] に含まれる値である必要がある。
    /// * `t` — 時間インデックス。
    ///
    /// # バリデーション
    ///
    /// - `i` が [`Self::TEMPORAL_I`] に含まれない場合、
    ///   [`Error::TIntervalError`](crate::SpatialIdError::TIntervalError) を返す。
    /// - 計算結果である `i * (t + 1) - 1` が `u64::MAX` を超える場合、
    ///   [`Error::TOutOfRange`](crate::SpatialIdError::TOutOfRange) を返す。
    ///
    /// # 例
    ///
    /// 有効な時間IDの作成:
    /// ```
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::TemporalId;
    /// let id = TemporalId::new(3600, 5).unwrap();
    /// assert_eq!(id.i(), 3600);
    /// assert_eq!(id.t(), 5);
    /// # }
    /// ```
    ///
    /// 無効な時間間隔の検知:
    /// ```
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::TemporalId;
    /// # use kasane_logic::Error;
    /// let id = TemporalId::new(7200, 5);
    /// assert!(id.is_err());
    /// # }
    /// ```
    pub fn new(i: u64, t: u64) -> Result<Self, Error> {
        Self::TEMPORAL_I
            .iter()
            .find(|&&interval| interval == i)
            .ok_or(SpatialIdError::TIntervalError { i })?;
        let inclusive_end = i as u128 * (t as u128) + i as u128 - 1;
        if inclusive_end > u64::MAX as u128 {
            return Err(SpatialIdError::TOutOfRange { i, t }.into());
        }
        Ok(Self { i, t })
    }

    /// このインスタンスが全時間を表す特別な値（`WHOLE`）であるかを判定する。
    ///
    /// `WHOLE` は `i = u64::MAX, t = 0` で、時間の制限がない状態を表す。
    ///
    /// # 戻り値
    ///
    /// 全時間を表す場合は `true`、そうでない場合は `false` を返す。
    ///
    /// # 例
    ///
    /// ```
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::TemporalId;
    /// let whole = TemporalId::WHOLE;
    /// assert!(whole.is_whole());
    ///
    /// let specific = TemporalId::new(3600, 5).unwrap();
    /// assert!(!specific.is_whole());
    /// # }
    /// ```
    pub fn is_whole(&self) -> bool {
        self.i == u64::MAX && self.t == 0
    }

    /// この時間区間の開始時刻をUNIXタイムスタンプ（秒単位）で取得する。
    ///
    /// 戻り値は `i * t` である。
    ///
    /// # 戻り値
    ///
    /// 時間区間の開始時刻（UNIXタイムスタンプ、秒単位）。
    ///
    /// # 例
    ///
    /// ```
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::TemporalId;
    /// let id = TemporalId::new(3600, 10).unwrap();
    /// assert_eq!(id.start_unixstamp(), 36000);
    /// # }
    /// ```
    pub fn start_unixstamp(&self) -> u64 {
        self.i * self.t
    }

    /// この時間区間の終了時刻をUNIXタイムスタンプ（秒単位、包括的）で取得する。
    ///
    /// 戻り値は `i * (t + 1) - 1` である。
    /// この値は時間区間に含まれる最後の秒を表す（包括的）。
    ///
    /// # 戻り値
    ///
    /// 時間区間の終了時刻（UNIXタイムスタンプ、秒単位、包括的）。
    ///
    /// # 例
    ///
    /// ```
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::TemporalId;
    /// let id = TemporalId::new(3600, 10).unwrap();
    /// assert_eq!(id.end_unixstamp_inclusive(), 39599);
    /// # }
    /// ```
    pub fn end_unixstamp_inclusive(&self) -> u64 {
        self.i * (self.t + 1) - 1
    }

    /// この時間区間の終了時刻をUNIXタイムスタンプ（秒単位、排他的）で取得する。
    ///
    /// 戻り値は `i * (t + 1)` である（`u128` 型）。
    /// この値は時間区間の次の秒を表す（排他的）。
    /// `u64::MAX` を超える可能性があるため、戻り値は `u128` 型である。
    ///
    /// # 戻り値
    ///
    /// 時間区間の終了時刻の次の秒（UNIXタイムスタンプ、秒単位、排他的、`u128`型）。
    ///
    /// # 例
    ///
    /// ```
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::TemporalId;
    /// let id = TemporalId::new(3600, 10).unwrap();
    /// assert_eq!(id.end_unixtime_exclusive(), 39600);
    /// # }
    /// ```
    pub fn end_unixtime_exclusive(&self) -> u128 {
        (self.i as u128) * ((self.t as u128) + 1)
    }

    /// 時間間隔 `i` を取得する。
    ///
    /// # 戻り値
    ///
    /// この [`TemporalId`] の時間間隔。
    ///
    /// # 例
    ///
    /// ```
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::TemporalId;
    /// let id = TemporalId::new(3600, 5).unwrap();
    /// assert_eq!(id.i(), 3600);
    /// # }
    /// ```
    pub fn i(&self) -> u64 {
        self.i
    }

    /// 時間インデックス `t` を取得する。
    ///
    /// # 戻り値
    ///
    /// この [`TemporalId`] の時間インデックス。
    ///
    /// # 例
    ///
    /// ```
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::TemporalId;
    /// let id = TemporalId::new(3600, 5).unwrap();
    /// assert_eq!(id.t(), 5);
    /// # }
    /// ```
    pub fn t(&self) -> u64 {
        self.t
    }

    /// 開始と終了（排他的）のUNIXタイムスタンプから、時間範囲を表す最小個数の [`TemporalId`] 列を生成する。
    ///
    /// 与えられた時間範囲 `[start, end_exclusive)` を表現するために、
    /// 最も大きな時間間隔から貪欲に [`TemporalId`] を切り出す。
    /// これにより、表現に必要な [`TemporalId`] の個数が最小となる。
    ///
    /// # パラメーター
    ///
    /// * `start` — 時間範囲の開始（UNIXタイムスタンプ、秒単位）
    /// * `end_exclusive` — 時間範囲の終了（UNIXタイムスタンプ、秒単位、排他的）
    ///
    /// # バリデーション
    ///
    /// - `start >= end_exclusive` の場合、エラーを返す。
    /// - 時間範囲の表現に失敗した場合（通常は発生しない）、エラーを返す。
    ///
    /// # 動作コスト
    ///
    /// 時間範囲の大きさに応じて変わるが、一般的には`O(k)` である。
    /// ここで `k` は生成される [`TemporalId`] の個数であり、通常は小さい値である。
    ///
    /// # 例
    ///
    /// 1時間の範囲:
    /// ```
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::TemporalId;
    /// let ids = TemporalId::from_range(0, 3600).unwrap();
    /// assert_eq!(ids.len(), 1);
    /// assert_eq!(ids[0], TemporalId::new(3600, 0).unwrap());
    /// # }
    /// ```
    ///
    /// 複雑な範囲（時間と分の組み合わせ）:
    /// ```
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::TemporalId;
    /// let ids = TemporalId::from_range(0, 3720).unwrap(); // 1時間 + 2分
    /// assert!(ids.len() >= 1);
    /// // 最初の要素は3600秒（1時間）の間隔を持つ
    /// # }
    /// ```
    ///
    /// 秒単位の細かい範囲:
    /// ```
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::TemporalId;
    /// let ids = TemporalId::from_range(100, 105).unwrap();
    /// assert_eq!(ids.len(), 5);
    /// // 5秒を1秒単位の5つのIDで表現
    /// # }
    /// ```
    pub fn from_range(start: u64, end_exclusive: u64) -> Result<Vec<TemporalId>, Error> {
        if start >= end_exclusive {
            return Err(SpatialIdError::TOutOfRange { i: 1, t: start }.into());
        }

        let mut result = Vec::new();
        let mut current = start;

        while current < end_exclusive {
            let remaining = end_exclusive - current;
            let mut placed = false;

            for &interval in &Self::TEMPORAL_I[1..] {
                if current.is_multiple_of(interval) && remaining >= interval {
                    let t = current / interval;
                    if let Ok(id) = TemporalId::new(interval, t) {
                        result.push(id);
                        current += interval;
                        placed = true;
                        break;
                    }
                }
            }

            if !placed {
                return Err(SpatialIdError::TOutOfRange { i: 1, t: current }.into());
            }
        }

        Ok(result)
    }
}
