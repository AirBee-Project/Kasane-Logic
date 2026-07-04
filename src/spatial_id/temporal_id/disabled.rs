use alloc::vec::Vec;

use core::fmt::{Display, Formatter};
use core::str::FromStr;

use crate::error::Error;

/// 時間ドメインの排他的終端（enabled 実装と同一値: `86400 × 2^47`）。
const DOMAIN_END: u64 = 86400 << 47;

#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord, Default)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
/// 時間IDの区間表現を表す型である（temporal_id feature無効時のスタブ）。
///
/// `temporal_id` feature が無効な場合、[`TemporalId`] は常に全時間を表す
/// スタブ実装となる。すべてのメソッドは全時間を表す状態を返す。
pub struct TemporalId;

impl TemporalId {
    /// 全時間を表す定数。
    pub const WHOLE: TemporalId = TemporalId;

    /// 時間ドメインの排他的終端（enabled 実装と同一値）。
    pub const DOMAIN_END: u64 = DOMAIN_END;

    /// 全ての時間を表す時間IDを作成する。
    ///
    /// `temporal_id` feature が無効な場合、常に `Ok(Self::WHOLE)` を返す。
    pub fn new(_i: u64, _t: u64) -> Result<Self, Error> {
        Ok(Self::WHOLE)
    }

    /// このインスタンスが全時間を表すかを判定する。
    ///
    /// `temporal_id` feature が無効な場合、常に `true` を返す。
    pub fn is_whole(&self) -> bool {
        true
    }

    /// 時間区間の開始時刻をUNIXタイムスタンプで取得する。
    ///
    /// `temporal_id` feature が無効な場合、常に `0` を返す。
    pub fn start_unixtime(&self) -> u64 {
        0
    }

    /// 時間区間の終了時刻（包括的）をUNIXタイムスタンプで取得する。
    ///
    /// `temporal_id` feature が無効な場合、常に `DOMAIN_END - 1` を返す。
    pub fn end_unixtime_inclusive(&self) -> u64 {
        DOMAIN_END - 1
    }

    /// 時間区間の終了時刻（排他的）をUNIXタイムスタンプで取得する。
    ///
    /// `temporal_id` feature が無効な場合、常に時間ドメイン終端 `DOMAIN_END` を返す。
    pub fn end_unixtime_exclusive(&self) -> u64 {
        DOMAIN_END
    }

    /// 時間間隔 `i` を取得する。
    ///
    /// `temporal_id` feature が無効な場合、常に `DOMAIN_END`（全時間の秒数）を返す。
    pub fn i(&self) -> u64 {
        DOMAIN_END
    }

    /// 時間インデックス `t` を取得する。
    ///
    /// `temporal_id` feature が無効な場合、常に `0` を返す。
    pub fn t(&self) -> u64 {
        0
    }

    /// 2つの時間IDの交差を計算する。
    ///
    /// `temporal_id` feature が無効な場合、常に全時間を返す。
    pub fn intersection(&self, _other: &TemporalId) -> Option<TemporalId> {
        Some(TemporalId::WHOLE)
    }

    /// `other` の時間範囲が `self` に完全に含まれるかを判定する。
    ///
    /// `temporal_id` feature が無効な場合、常に `true` を返す。
    pub fn contains(&self, _other: &TemporalId) -> bool {
        true
    }

    /// 開始と終了から複数のTemporalIdを生成する。
    ///
    /// `temporal_id` feature が無効な場合、常に全時間を表す1つの要素を含むベクトルを返す。
    pub fn from_range(_start: u64, _end_exclusive: u64) -> Result<Vec<TemporalId>, Error> {
        Ok(vec![Self::WHOLE])
    }

    /// 2つの時間IDの差集合を計算する。
    ///
    /// `temporal_id` feature が無効な場合、常に空（WHOLE − WHOLE = 空）を返す。
    pub fn difference(&self, other: &TemporalId) -> impl Iterator<Item = TemporalId> {
        let _ = other;
        core::iter::empty()
    }
}

impl Display for TemporalId {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}/0", DOMAIN_END)
    }
}

impl FromStr for TemporalId {
    type Err = Error;
    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        Ok(Self::WHOLE)
    }
}

/// カレンダー時間の集合（temporal_id feature無効時のスタブ）。
///
/// feature 無効時はすべての時間IDが全時間（WHOLE）なので、集合は
/// 「空」か「全時間」の2状態のみをとる。[`SpatialIdSet`](crate::SpatialIdSet) の
/// 葉の値として、有効時の [`TemporalSet`](crate::TemporalSet) と同じ最小APIを提供する。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct TemporalSet {
    whole: bool,
}

impl TemporalSet {
    /// 空集合を作る。
    pub fn new() -> Self {
        Self { whole: false }
    }

    /// 全時間（WHOLE）の集合を作る。
    pub fn whole() -> Self {
        Self { whole: true }
    }

    /// 全時間（WHOLE）と等しいか。
    pub fn is_whole(&self) -> bool {
        self.whole
    }

    /// 1つの [`TemporalId`] が覆う時間の集合を作る（feature 無効時は常に全時間）。
    pub fn from_temporal(_t: &TemporalId) -> Self {
        Self::whole()
    }

    /// [`TemporalId`] を集合へ追加する（union）。
    pub fn insert(&mut self, _t: &TemporalId) {
        self.whole = true;
    }

    /// 空かどうか。
    pub fn is_empty(&self) -> bool {
        !self.whole
    }

    /// 指定の UNIX 秒が含まれるか。
    pub fn contains_unixtime(&self, _sec: u64) -> bool {
        self.whole
    }

    /// `t` の時間範囲が完全に含まれるか。
    pub fn contains(&self, _t: &TemporalId) -> bool {
        self.whole
    }

    /// 和集合。
    pub fn union(&self, other: &Self) -> Self {
        Self {
            whole: self.whole || other.whole,
        }
    }

    /// 積集合。
    pub fn intersection(&self, other: &Self) -> Self {
        Self {
            whole: self.whole && other.whole,
        }
    }

    /// 差集合 `self - other`。
    pub fn difference(&self, other: &Self) -> Self {
        Self {
            whole: self.whole && !other.whole,
        }
    }

    /// 集合をセル列へ分解する（feature 無効時は WHOLE 1個か空）。
    pub fn cells(&self) -> Vec<TemporalId> {
        if self.whole {
            vec![TemporalId::WHOLE]
        } else {
            Vec::new()
        }
    }

    /// `window` に限定したセル列を返す。
    pub fn cells_in_window(&self, window: &TemporalId) -> Vec<TemporalId> {
        self.intersection(&Self::from_temporal(window)).cells()
    }
}
