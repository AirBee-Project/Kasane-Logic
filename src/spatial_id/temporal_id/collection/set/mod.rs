use alloc::vec::Vec;
use core::ops::{BitAnd, BitOr, Sub};

use super::temporal_core::TemporalCore;
use crate::TemporalId;

/// [`TemporalId`]の集合を表す型。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct TemporalSet(pub(crate) TemporalCore<()>);

impl TemporalSet {
    /// 空集合を作る。
    pub fn new() -> Self {
        Self(TemporalCore::new())
    }

    /// 全ての時間を表すの集合を作成する。
    pub fn whole() -> Self {
        Self(TemporalCore::from_segment(
            0,
            crate::Interval::WHOLE_SECONDS,
            (),
        ))
    }

    /// この集合は全ての時間を表しているかを判定する。
    pub fn is_whole(&self) -> bool {
        self.0.segments() == [(0, crate::Interval::WHOLE_SECONDS, ())]
    }

    /// 1つの [`TemporalId`] が覆う時間の集合を作る。
    pub fn from_temporal(t: &TemporalId) -> Self {
        Self(TemporalCore::from_temporal(t, ()))
    }

    /// [`TemporalId`] を集合へ追加する（union）。
    pub fn insert(&mut self, t: &TemporalId) {
        *self = self.union(&Self::from_temporal(t));
    }

    /// 空かどうか。
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// 正規化済み区間列 `[start, end)` を所有権付きで返す（クレート内部・テスト用フック）。
    #[cfg(test)]
    pub(crate) fn intervals(&self) -> Vec<(u64, u64)> {
        self.0.segments().iter().map(|&(s, e, ())| (s, e)).collect()
    }

    /// 指定の UNIX 秒が含まれるか（二分探索）。
    pub fn contains_unixtime(&self, sec: u64) -> bool {
        self.0.value_at(sec).is_some()
    }

    /// `t` の時間範囲が完全に含まれるか（`t ⊆ self`）。
    pub fn contains(&self, t: &TemporalId) -> bool {
        Self::from_temporal(t).difference(self).is_empty()
    }

    /// 和集合。
    pub fn union(&self, other: &Self) -> Self {
        Self(
            self.0
                .sweep(&other.0, |a, b| (a.is_some() || b.is_some()).then_some(())),
        )
    }

    /// 積集合。
    pub fn intersection(&self, other: &Self) -> Self {
        Self(
            self.0
                .sweep(&other.0, |a, b| (a.is_some() && b.is_some()).then_some(())),
        )
    }

    /// 差集合 `self - other`。
    pub fn difference(&self, other: &Self) -> Self {
        Self(self.0.difference(&other.0))
    }

    /// 集合を約数鎖の最小セル列（[`TemporalId`]）へ分解する。
    ///
    /// 約数鎖に二進層（`Day·2^k`）があるため、どの区間も高々数百セルに収まる
    /// （ドメイン全体は単一の [`TemporalId::WHOLE`] になる）。
    pub fn cells(&self) -> Vec<TemporalId> {
        self.0.cells().into_iter().map(|(t, ())| t).collect()
    }

    /// `window` に限定したセル列を返す（`(self ∩ window)` の分解）。
    pub fn cells_clipped(&self, window: &TemporalId) -> Vec<TemporalId> {
        self.intersection(&Self::from_temporal(window)).cells()
    }
}

impl From<&TemporalId> for TemporalSet {
    fn from(t: &TemporalId) -> Self {
        Self::from_temporal(t)
    }
}

impl From<TemporalId> for TemporalSet {
    fn from(t: TemporalId) -> Self {
        Self::from_temporal(&t)
    }
}

impl BitOr for &TemporalSet {
    type Output = TemporalSet;
    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

impl BitAnd for &TemporalSet {
    type Output = TemporalSet;
    fn bitand(self, rhs: Self) -> Self::Output {
        self.intersection(rhs)
    }
}

impl Sub for &TemporalSet {
    type Output = TemporalSet;
    fn sub(self, rhs: Self) -> Self::Output {
        self.difference(rhs)
    }
}

impl IntoIterator for &TemporalSet {
    type Item = TemporalId;
    type IntoIter = alloc::vec::IntoIter<TemporalId>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells().into_iter()
    }
}

#[cfg(test)]
mod tests;
