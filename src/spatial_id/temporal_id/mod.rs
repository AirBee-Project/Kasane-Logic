#[cfg(not(feature = "temporal_id"))]
mod disabled;
#[cfg(not(feature = "temporal_id"))]
pub use disabled::TemporalId;

#[cfg(feature = "temporal_id")]
use crate::{Interval, error::Error};
#[cfg(feature = "temporal_id")]
pub mod impls;

pub mod interval;

#[cfg(feature = "temporal_id")]
#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
/// 時間IDの区間表現を表す型である。
pub struct TemporalId {
    /// 時間間隔。
    i: Interval,
    /// 時間インデックス。
    t: u64,
}

#[cfg(feature = "temporal_id")]
impl TemporalId {
    /// 指定された時間間隔と時間インデックスから新しい [`TemporalId`] を構築する。
    ///
    /// 与えられた `i` と `t` が有効な値であるかを検証し、
    /// 検証に失敗した場合は [`Error`] を返す。
    ///
    /// # パラメーター
    ///
    /// * `i` — 時間間隔（秒単位）。[`Self::TEMPORAL_I`] に含まれる値である必要がある。
    /// * `t` — 時間インデックス。
    pub fn new<I:Into<u64>>(i: I, t: u64) -> Result<Self, Error> {
        todo!()
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
        self.i ==Interval::Whole && self.t == 0
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
    pub fn i(&self) -> Interval {
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
}
