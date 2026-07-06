use alloc::vec::Vec;

use core::fmt::{Display, Formatter};
use core::str::FromStr;

use crate::error::Error;

/// 時間ドメインの排他的終端。
const DOMAIN_END: u64 = 86400 << 47;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
/// 時間IDの時間間隔`i`を表現する型。
pub enum Interval {
    #[default]
    Whole,
    /// `86400 × 2^k` 秒（`k = 1..=46`）
    #[non_exhaustive]
    DayPow { k: u8 },
    /// 1日（86400 秒）
    Day,
    /// 1時間（3600 秒）
    Hour,
    /// 1分（60 秒）
    Minute,
    /// 1秒（1 秒）
    Second,
}

impl Interval {
    /// このライブラリが扱える全時間の秒数。86400 × 2^47`（約3,850億年）。
    pub const WHOLE_SECONDS: u64 = 86400 << 47;

    /// 最も粗い時間区間を表す二進層の指数。
    pub const WHOLE_POW: u8 = 47;

    /// 秒数から[Interval]型を作成する。
    pub fn new(seconds: u64) -> Result<Interval, Error> {
        match seconds {
            Self::WHOLE_SECONDS => Ok(Interval::Whole),
            86400 => Ok(Interval::Day),
            3600 => Ok(Interval::Hour),
            60 => Ok(Interval::Minute),
            1 => Ok(Interval::Second),
            _ => Ok(Interval::Whole),
        }
    }

    /// 二進層の間隔 `Day·2^k` を作成する。
    pub fn day_pow(k: u8) -> Result<Interval, Error> {
        match k {
            0 => Ok(Interval::Day),
            1..=46 => Ok(Interval::DayPow { k }),
            Self::WHOLE_POW => Ok(Interval::Whole),
            _ => Ok(Interval::Whole),
        }
    }

    /// この間隔の秒数。
    pub const fn seconds(self) -> u64 {
        match self {
            Interval::Whole => Self::WHOLE_SECONDS,
            Interval::DayPow { k } => 86400 << k,
            Interval::Day => 86400,
            Interval::Hour => 3600,
            Interval::Minute => 60,
            Interval::Second => 1,
        }
    }
}

impl TryFrom<u64> for Interval {
    type Error = Error;
    fn try_from(seconds: u64) -> Result<Self, Self::Error> {
        Self::new(seconds)
    }
}

impl TryFrom<i32> for Interval {
    type Error = Error;
    fn try_from(seconds: i32) -> Result<Self, Self::Error> {
        if seconds < 0 {
            return Err(crate::SpatialIdError::TIntervalError { i: seconds as u64 }.into());
        }
        Self::try_from(seconds as u64)
    }
}

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

    /// 新しい [`TemporalId`] を作成する。
    pub fn new<I>(_interval: I, _t: u64) -> Result<Self, Error>
    where
        I: TryInto<Interval>,
        Error: From<I::Error>,
    {
        let _ = _interval.try_into()?;
        Ok(Self::WHOLE)
    }

    /// 全ての時間を表す時間IDを作成する。
    ///
    /// `temporal_id` feature が無効な場合、常に `Ok(Self::WHOLE)` を返す。
    pub fn from_seconds(_i: u64, _t: u64) -> Result<Self, Error> {
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

    /// 時間区間の終了時刻（排他的）をUNIXタイムスタンプで取得する。
    ///
    /// `temporal_id` feature が無効な場合、常に時間ドメイン終端 `DOMAIN_END` を返す。
    pub fn end_unixtime_exclusive(&self) -> u64 {
        DOMAIN_END
    }

    /// 時間間隔を取得する。
    ///
    /// `temporal_id` feature が無効な場合、常に `Interval::Whole` を返す。
    pub fn i(&self) -> Interval {
        Interval::Whole
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
        if _start >= _end_exclusive {
            return Ok(vec![]);
        }
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

    /// 正規化済み区間列を返す（クレート内部の走査用フック）。
    pub(crate) fn intervals(&self) -> &[(u64, u64)] {
        if self.whole { &[(0, DOMAIN_END)] } else { &[] }
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

/// 時間 → 値 `V` の対応（temporal_id feature無効時のスタブ）。
///
/// feature 無効時はすべての時間IDが全時間（WHOLE）なので、マップは
/// 「空」か「全時間 → 1つの値」の2状態のみをとる。
/// [`SpatialIdMap`](crate::SpatialIdMap) / [`SpatialIdTable`](crate::SpatialIdTable) の
/// 葉の値として、有効時の [`TemporalMap`](crate::TemporalMap) と同じ最小APIを提供する。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct TemporalMap<V> {
    value: Option<V>,
}

impl<V: Clone + PartialEq> TemporalMap<V> {
    /// 空。
    pub fn new() -> Self {
        Self { value: None }
    }

    /// 1つの [`TemporalId`] に値 `v` を対応させる（feature 無効時は常に全時間）。
    pub fn from_temporal(_t: &TemporalId, v: V) -> Self {
        Self { value: Some(v) }
    }

    /// 空かどうか。
    pub fn is_empty(&self) -> bool {
        self.value.is_none()
    }

    /// 指定秒の値。
    pub fn value_at(&self, _sec: u64) -> Option<&V> {
        self.value.as_ref()
    }

    /// 上書き合成（other が存在すれば other が勝つ）。
    pub fn overwrite(&self, other: &Self) -> Self {
        Self {
            value: other.value.clone().or_else(|| self.value.clone()),
        }
    }

    /// 差集合 `self - other`（時間で other を除く）。
    pub fn difference(&self, other: &Self) -> Self {
        Self {
            value: if other.value.is_some() {
                None
            } else {
                self.value.clone()
            },
        }
    }

    /// 時間集合 `set` に含まれる時間だけを残す。
    pub fn intersect_time(&self, set: &TemporalSet) -> Self {
        Self {
            value: if set.is_whole() {
                self.value.clone()
            } else {
                None
            },
        }
    }

    /// 時間集合 `set` に含まれる時間を取り除く。
    pub fn subtract_time(&self, set: &TemporalSet) -> Self {
        Self {
            value: if set.is_empty() {
                self.value.clone()
            } else {
                None
            },
        }
    }

    /// 全セグメントをセル列 `(TemporalId, V)` へ分解する。
    pub fn cells(&self) -> Vec<(TemporalId, V)> {
        self.value
            .iter()
            .map(|v| (TemporalId::WHOLE, v.clone()))
            .collect()
    }

    /// [`cells`](Self::cells) の参照版。
    pub fn cells_ref(&self) -> Vec<(TemporalId, &V)> {
        self.value.iter().map(|v| (TemporalId::WHOLE, v)).collect()
    }

    /// `window` に限定したセル列を参照で返す（feature 無効時は全時間のみ）。
    pub fn cells_in_window_ref(&self, _window: &TemporalId) -> Vec<(TemporalId, &V)> {
        self.cells_ref()
    }

    /// 正規化済みセグメント列 `(start, end, &V)` を返す（永続化・走査用の内部フック）。
    pub(crate) fn segments_ref(&self) -> Vec<(u64, u64, &V)> {
        self.value.iter().map(|v| (0, DOMAIN_END, v)).collect()
    }

    /// セグメント列から構築する（永続化復元用の内部フック）。
    pub(crate) fn from_raw_segments(mut segments: Vec<(u64, u64, V)>) -> Self {
        Self {
            value: segments.pop().map(|(_, _, v)| v),
        }
    }
}
