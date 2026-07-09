// temporal_id feature 無効時は専用のスタブを使う。
#[cfg(not(feature = "temporal_id"))]
mod disabled;
#[cfg(not(feature = "temporal_id"))]
pub use disabled::TemporalSet;

#[cfg(feature = "temporal_id")]
use crate::{TemporalId, spatial_id::temporal_id::collection::core::TemporalCore};
#[cfg(feature = "temporal_id")]
pub mod impls;
#[cfg(all(test, feature = "temporal_id"))]
mod tests;

/// [`TemporalId`]の集合を表す型。
#[cfg(feature = "temporal_id")]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct TemporalSet(pub(crate) TemporalCore<()>);

#[cfg(feature = "temporal_id")]
impl TemporalSet {
    /// 空集合を作る。
    pub fn new() -> Self {
        Self(TemporalCore::new())
    }

    /// 全ての時間を表すの集合を作成する。
    pub fn whole() -> Self {
        let mut core: TemporalCore<()> = TemporalCore::new();
        core.insert(0..crate::Interval::WHOLE_SECONDS, ());
        Self(core)
    }

    /// この集合は全ての時間を表しているかを判定する。
    pub fn is_whole(&self) -> bool {
        self.0.ranges() == [(0, crate::Interval::WHOLE_SECONDS, ())]
    }

    /// [`TemporalId`] を集合へ追加する。
    pub fn insert(&mut self, t: &TemporalId) {
        self.0
            .insert(t.start_unixtime()..t.end_unixtime_exclusive(), ());
    }

    /// その時刻が含まれるか
    pub fn contains_unixtime(&self, sec: u64) -> bool {
        self.0.contains_unixtime_range(sec).is_some()
    }

    pub fn get(&self, target: TemporalId) -> impl Iterator<Item = TemporalId> + '_ {
        let (w0, w1) = (target.start_unixtime(), target.end_unixtime_exclusive());
        self.0.ranges().iter().flat_map(move |(s, e, _)| {
            let cs = (*s).max(w0);
            let ce = (*e).min(w1);
            TemporalId::from_range(cs..ce).unwrap()
        })
    }

    /// 空かどうか。
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// 正規化済み区間列 `[start, end)` を所有権付きで返す
    #[cfg(test)]
    pub(crate) fn ranges(&self) -> Vec<(u64, u64)> {
        self.0.ranges().iter().map(|&(s, e, ())| (s, e)).collect()
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

    /// 保持する[TemporalId]の個数を返します。
    pub fn len(&self) -> usize {
        self.0.len()
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
