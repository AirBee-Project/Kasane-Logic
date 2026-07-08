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
        Self(TemporalCore {
            ranges: alloc::vec![(0, crate::Interval::WHOLE_SECONDS, ())],
            cached_len: 1,
        })
    }

    /// この集合は全ての時間を表しているかを判定する。
    pub fn is_whole(&self) -> bool {
        self.0.ranges() == [(0, crate::Interval::WHOLE_SECONDS, ())]
    }

    /// [`TemporalId`] を集合へ追加する（union）。
    pub fn insert(&mut self, t: &TemporalId) {
        self.0
            .insert(t.start_unixtime()..t.end_unixtime_exclusive(), ());
    }

    /// 空かどうか。
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// 正規化済み区間列 `[start, end)` を所有権付きで返す（クレート内部・テスト用フック）。
    #[cfg(test)]
    pub(crate) fn intervals(&self) -> Vec<(u64, u64)> {
        self.0.ranges().iter().map(|&(s, e, ())| (s, e)).collect()
    }

    /// 指定の UNIX 秒が含まれるか（二分探索）。
    pub fn contains_unixtime(&self, sec: u64) -> bool {
        self.0.get(sec).is_some()
    }

    /// `t` の時間範囲が完全に含まれるか（`t ⊆ self`）。
    pub fn contains(&self, t: &TemporalId) -> bool {
        Self::from(t).difference(self).is_empty()
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

    /// 保持する時間セルの総数を返します（O(1)）。
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// `TemporalSet` の時間セルを走査するイテレータを返します。
    pub fn iter(&self) -> impl Iterator<Item = TemporalId> + '_ {
        self.0.iter().map(|(t, ())| t)
    }

    /// 指定された時間範囲を集合から取り除きます。
    pub fn remove(&mut self, t: &TemporalId) {
        *self = self.difference(&Self::from(t));
    }

    /// 集合をクリアして空にします。
    pub fn clear(&mut self) {
        self.0 = TemporalCore::new();
    }
}

impl From<&TemporalId> for TemporalSet {
    fn from(t: &TemporalId) -> Self {
        let mut set = Self::new();
        set.insert(t);
        set
    }
}

impl From<TemporalId> for TemporalSet {
    fn from(t: TemporalId) -> Self {
        let mut set = Self::new();
        set.insert(&t);
        set
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
        self.iter().collect::<Vec<_>>().into_iter()
    }
}

#[cfg(test)]
mod tests;
