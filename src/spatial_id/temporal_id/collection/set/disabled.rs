use crate::TemporalId;

const DOMAIN_END: u64 = crate::Interval::WHOLE_SECONDS;

/// 「空」か「全時間」の 2 状態のみをとる。
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

    /// この集合は全ての時間を表しているかを判定する。
    pub fn is_whole(&self) -> bool {
        self.whole
    }

    /// [`TemporalId`] を集合へ追加する。
    pub fn insert(&mut self, _t: &TemporalId) {
        self.whole = true;
    }

    /// 空かどうか。
    pub fn is_empty(&self) -> bool {
        !self.whole
    }

    /// その時刻が含まれるか。
    pub fn contains_unixtime(&self, _sec: u64) -> bool {
        self.whole
    }

    /// `target` の時間範囲と交差する [`TemporalId`] を返す。
    ///
    /// `temporal_id` feature 無効時、WHOLE のみなので交差があれば WHOLE を返す。
    pub fn get(&self, _target: TemporalId) -> impl Iterator<Item = TemporalId> + '_ {
        let opt = self.whole.then_some(TemporalId::WHOLE);
        opt.into_iter()
    }

    /// `t` の時間範囲が完全に含まれるか。
    pub fn contains(&self, _t: &TemporalId) -> bool {
        self.whole
    }

    /// 正規化済み区間列を返す（クレート内部の走査用フック）。
    #[allow(dead_code)]
    pub(crate) fn intervals(&self) -> &[(u64, u64)] {
        if self.whole { &[(0, DOMAIN_END)] } else { &[] }
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

    /// 保持する [`TemporalId`] の個数を返す。
    pub fn len(&self) -> usize {
        if self.whole { 1 } else { 0 }
    }

    /// 指定された時間範囲を集合から取り除く。
    pub fn remove(&mut self, _t: &TemporalId) {
        self.whole = false;
    }

    /// 集合をクリアして空にする。
    pub fn clear(&mut self) {
        self.whole = false;
    }

    /// 保持する [`TemporalId`] を走査するイテレータを返す。
    pub fn iter(&self) -> impl Iterator<Item = TemporalId> + '_ {
        let opt = self.whole.then_some(TemporalId::WHOLE);
        opt.into_iter()
    }
}

// ── traitImpl ─────────────────────────────────────────────────────────────────

impl From<&TemporalId> for TemporalSet {
    fn from(_t: &TemporalId) -> Self {
        Self::whole()
    }
}

impl From<TemporalId> for TemporalSet {
    fn from(_t: TemporalId) -> Self {
        Self::whole()
    }
}

impl<'a> IntoIterator for &'a TemporalSet {
    type Item = TemporalId;
    type IntoIter = core::option::IntoIter<TemporalId>;

    fn into_iter(self) -> Self::IntoIter {
        self.whole.then_some(TemporalId::WHOLE).into_iter()
    }
}

impl IntoIterator for TemporalSet {
    type Item = TemporalId;
    type IntoIter = core::option::IntoIter<TemporalId>;

    fn into_iter(self) -> Self::IntoIter {
        self.whole.then_some(TemporalId::WHOLE).into_iter()
    }
}

use core::ops::{BitAnd, BitOr, Sub};

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
