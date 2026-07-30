use core::fmt::{Display, Formatter};
use core::str::FromStr;

use crate::error::Error;

/// `temporal_id` feature 無効時の時間表現。常に全時間を表すスタブ。
///
/// feature 有効時は[`TemporalSegment`](crate::spatial_id::temporal_id::TemporalSegment)
/// （FlexTree内部専用の生セル）と`TemporalId`（人間向け公開表現）が別の型だが、値が常に1種類
/// （全時間）しか無い無効時はこの単一の型へ両方が解決される。
#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord, Default)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct TemporalId;

/// `TemporalSegment` はこのスタブへのクレート内部専用エイリアス（feature 無効時は区別しない）。
pub(crate) use self::TemporalId as TemporalSegment;

impl TemporalId {
    /// 全時間を表す定数。
    pub const WHOLE: TemporalId = TemporalId;

    /// このインスタンスが全時間を表すかを判定する。常に `true`。
    pub fn is_whole(&self) -> bool {
        true
    }

    /// 時間軸のズームレベル。`temporal_id` feature 無効時は常に `0`。
    pub(crate) fn zoom(&self) -> u8 {
        0
    }

    /// 時間軸のインデックス。`temporal_id` feature 無効時は常に `0`。
    pub(crate) fn index(&self) -> u64 {
        0
    }

    /// `temporal_id` feature 無効時は分割できないため常に `None`。
    pub(crate) fn split(&self, _side: crate::Side) -> Option<Self> {
        None
    }

    /// 2つの時間IDの交差を計算する。両者が全時間ならば全時間を返す。
    pub(crate) fn intersection(&self, _other: &Self) -> Option<Self> {
        Some(Self)
    }

    /// 2つの時間IDの差集合を計算する。`temporal_id` feature 無効時は空のイテレータを返す。
    pub(crate) fn difference(&self, _other: &Self) -> impl Iterator<Item = Self> {
        core::iter::empty()
    }

    /// この範囲を生セルの列へ分解する。`temporal_id` feature 無効時は常に自身1個。
    pub(crate) fn segments(&self) -> impl Iterator<Item = Self> {
        core::iter::once(Self)
    }

    /// この時間区間の単位。`temporal_id` feature 無効時は常に[`Interval::Whole`](crate::Interval::Whole)。
    pub fn interval(&self) -> crate::Interval {
        crate::Interval::Whole
    }

    /// この時間区間のインデックス範囲。`temporal_id` feature 無効時は常に `[0, 0]`。
    pub fn t(&self) -> [u64; 2] {
        [0, 0]
    }
}

impl From<&TemporalId> for TemporalId {
    fn from(value: &TemporalId) -> Self {
        value.clone()
    }
}

impl Display for TemporalId {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "0/0")
    }
}

impl FromStr for TemporalId {
    type Err = Error;
    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        Ok(Self::WHOLE)
    }
}
