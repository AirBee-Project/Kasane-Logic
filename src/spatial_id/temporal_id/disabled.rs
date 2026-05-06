use core::fmt::{Display, Formatter};
use std::str::FromStr;

use crate::error::Error;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
/// 時間IDの区間表現を表す型である（temporal_id feature無効時のスタブ）。
///
/// `temporal_id` feature が無効な場合、[`TemporalId`] は常に全時間を表す
/// スタブ実装となる。すべてのメソッドは全時間を表す状態を返す。
pub struct TemporalId;

impl TemporalId {
    /// 全時間を表す定数。
    pub const WHOLE: TemporalId = TemporalId;

    /// 時間IDを構築する（スタブ）。
    ///
    /// `temporal_id` feature が無効な場合、常に `Ok(Self::WHOLE)` を返す。
    pub fn new(_i: u64, _t: u64) -> Result<Self, Error> {
        Ok(Self::WHOLE)
    }

    /// このインスタンスが全時間を表すかを判定する（スタブ）。
    ///
    /// `temporal_id` feature が無効な場合、常に `true` を返す。
    pub fn is_whole(&self) -> bool {
        true
    }

    /// 時間区間の開始時刻をUNIXタイムスタンプで取得する（スタブ）。
    ///
    /// `temporal_id` feature が無効な場合、常に `0` を返す。
    pub fn start_unixstamp(&self) -> u64 {
        0
    }

    /// 時間区間の終了時刻をUNIXタイムスタンプで取得する（包括的、スタブ）。
    ///
    /// `temporal_id` feature が無効な場合、常に `u64::MAX` を返す。
    pub fn end_unixstamp_inclusive(&self) -> u64 {
        u64::MAX
    }

    /// 時間区間の終了時刻をUNIXタイムスタンプで取得する（排他的、スタブ）。
    ///
    /// `temporal_id` feature が無効な場合、常に `u64::MAX + 1` を返す。
    pub fn end_unixtime_exclusive(&self) -> u128 {
        (u64::MAX as u128) + 1
    }

    /// 時間間隔 `i` を取得する（スタブ）。
    ///
    /// `temporal_id` feature が無効な場合、常に `u64::MAX` を返す。
    pub fn i(&self) -> u64 {
        u64::MAX
    }

    /// 時間インデックス `t` を取得する（スタブ）。
    ///
    /// `temporal_id` feature が無効な場合、常に `0` を返す。
    pub fn t(&self) -> u64 {
        0
    }

    /// 2つの時間IDの交差を計算する（スタブ）。
    ///
    /// `temporal_id` feature が無効な場合、両者が全時間ならば全時間を返す。
    pub fn intersection(&self, other: &TemporalId) -> Option<TemporalId> {
        if self.is_whole() && other.is_whole() {
            Some(TemporalId::WHOLE)
        } else {
            None
        }
    }

    /// 開始と終了から複数のTemporalIdを生成する（スタブ）。
    ///
    /// `temporal_id` feature が無効な場合、常に全時間を表す1つの要素を含むベクトルを返す。
    pub fn from_range(_start: u64, _end_exclusive: u64) -> Result<Vec<TemporalId>, Error> {
        Ok(vec![Self::WHOLE])
    }

    /// 2つの時間IDの差集合を計算する（スタブ）。
    ///
    /// `temporal_id` feature が無効な場合、空のイテレータを返す。
    pub fn difference(&self, other: &TemporalId) -> impl Iterator<Item = TemporalId> {
        let _ = other;
        std::iter::empty()
    }
}

impl Display for TemporalId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("18446744073709551615/0")
    }
}

impl FromStr for TemporalId {
    type Err = Error;
    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        Ok(Self::WHOLE)
    }
}
