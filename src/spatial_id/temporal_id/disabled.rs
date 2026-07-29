use alloc::vec;
use alloc::vec::Vec;
use core::fmt::{Display, Formatter};
use core::str::FromStr;

use crate::error::Error;
use crate::spatial_id::traits::TemporalId as TemporalIdTrait;

/// `temporal_id` feature 無効時の時間表現。常に全時間を表すスタブ。
///
/// [`TemporalCell`]（生セル）と[`TemporalRange`]（人間向け範囲）の
/// 両方が、この単一の型へのエイリアスとして解決される。値が常に1種類（全時間）しか無いため、
/// 2つの表現を分ける意味が無い。
#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord, Default)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct TemporalCell;

/// `TemporalRange` はこのスタブへのエイリアス（feature 無効時は区別しない）。
pub type TemporalRange = TemporalCell;

impl TemporalCell {
    /// 全時間を表す定数。
    pub const WHOLE: TemporalCell = TemporalCell;

    /// このインスタンスが全時間を表すかを判定する。常に `true`。
    pub fn is_whole(&self) -> bool {
        true
    }

    /// 時間軸のズームレベル。`temporal_id` feature 無効時は常に `0`。
    pub fn zoom(&self) -> u8 {
        0
    }

    /// 時間軸のインデックス。`temporal_id` feature 無効時は常に `0`。
    pub fn index(&self) -> u64 {
        0
    }

    /// `temporal_id` feature 無効時は分割できないため常に `None`。
    pub fn split(&self, _side: crate::Side) -> Option<Self> {
        None
    }

    /// 2つの時間IDの交差を計算する。両者が全時間ならば全時間を返す。
    pub fn intersection(&self, _other: &Self) -> Option<Self> {
        Some(Self)
    }

    /// 2つの時間IDの差集合を計算する。`temporal_id` feature 無効時は空のイテレータを返す。
    pub fn difference(&self, _other: &Self) -> impl Iterator<Item = Self> {
        core::iter::empty()
    }

    /// この範囲を生セルの列へ分解する。`temporal_id` feature 無効時は常に自身1個。
    pub fn into_cells(&self) -> impl Iterator<Item = Self> {
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

    /// 開始と終了から複数の時間IDを生成する。`temporal_id` feature 無効時は常に全時間を表す1つの要素を含むベクトルを返す。
    pub fn from_range(_start: u64, _end_exclusive: u64) -> Result<Vec<Self>, Error> {
        Ok(vec![Self::WHOLE])
    }
}

impl TemporalIdTrait for TemporalCell {
    const WHOLE: Self = TemporalCell;

    fn is_whole(&self) -> bool {
        true
    }

    fn seconds_range(&self) -> (u64, u64) {
        (0, crate::Interval::WHOLE_SECONDS)
    }
}

impl Display for TemporalCell {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "0/0")
    }
}

impl FromStr for TemporalCell {
    type Err = Error;
    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        Ok(Self::WHOLE)
    }
}
