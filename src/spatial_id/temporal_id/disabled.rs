use crate::{Interval, error::Error};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct TemporalId;

impl TemporalId {
    /// 全時間を表す定数。
    pub const WHOLE: TemporalId = TemporalId;

    /// 新しい [`TemporalId`] を作成する。
    ///
    /// `temporal_id` feature 無効時は [`Interval::WHOLE_SECONDS`] のみ受け付ける。
    pub fn new<I>(_interval: I, _t: u64) -> Result<Self, Error>
    where
        I: TryInto<Interval>,
        Error: From<I::Error>,
    {
        let _ = _interval.try_into()?;
        Ok(Self::WHOLE)
    }

    /// このインスタンスが全時間を表すかを判定する。
    ///
    /// `temporal_id` feature 無効時は常に `true` を返す。
    pub fn is_whole(&self) -> bool {
        true
    }

    /// 時間区間の開始時刻をUNIXタイムスタンプで取得する。
    ///
    /// `temporal_id` feature 無効時は常に `0` を返す。
    pub fn start_unixtime(&self) -> u64 {
        0
    }

    /// 時間区間の終了時刻（排他的）をUNIXタイムスタンプで取得する。
    ///
    /// `temporal_id` feature 無効時は常に時間ドメイン終端 [`Interval::WHOLE_SECONDS`] を返す。
    pub fn end_unixtime_exclusive(&self) -> u64 {
        Interval::WHOLE_SECONDS
    }

    /// 時間間隔を取得する。
    ///
    /// `temporal_id` feature 無効時は常に [`Interval::Whole`] を返す。
    pub fn i(&self) -> Interval {
        Interval::Whole
    }

    /// 時間インデックス `t` を取得する。
    ///
    /// `temporal_id` feature 無効時は常に `0` を返す。
    pub fn t(&self) -> u64 {
        0
    }

    /// 開始と終了の UNIX タイムスタンプから [`TemporalId`] のイテレータを生成する。
    ///
    /// `temporal_id` feature 無効時は有効な範囲なら `WHOLE` を 1 つ返す。
    pub fn from_range(
        range: core::ops::Range<u64>,
    ) -> Result<impl Iterator<Item = TemporalId>, Error> {
        let mut yielded = false;
        let empty = range.start >= range.end;
        Ok(core::iter::from_fn(move || {
            if empty || yielded {
                return None;
            }
            yielded = true;
            Some(TemporalId::WHOLE)
        }))
    }

    /// 開始と終了のUNIXタイムスタンプから、時間範囲を表す [`TemporalId`] の個数を返す。
    #[allow(dead_code)]
    pub(crate) fn count_range(range: core::ops::Range<u64>) -> usize {
        Self::from_range(range).unwrap().count()
    }
}
