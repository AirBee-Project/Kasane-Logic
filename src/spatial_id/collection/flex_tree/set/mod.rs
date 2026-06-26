use hashbrown::HashSet;

use crate::{FlexId, FlexTreeCore, IntoSingleIds, IterFlexIds, RangeId, SingleId, SpatialId};
pub mod convert;
pub mod json;
pub mod ops;
pub mod tests;

#[cfg(all(test, feature = "persist"))]
mod persist_tests {
    use super::SpatialIdSet;
    use crate::{RangeId, SingleId};

    /// FlexId をそのまま比較する（独自 PartialEq の zoom 正規化展開は使わない）。
    fn sorted(set: &SpatialIdSet) -> Vec<crate::FlexId> {
        let mut v: Vec<_> = set.iter().collect();
        v.sort();
        v
    }

    #[test]
    fn round_trip() {
        let mut set = SpatialIdSet::new();
        set.insert(SingleId::new(20, 0, 0, 0).unwrap());
        set.insert(SingleId::new(18, 1, 5, 7).unwrap());
        set.insert(RangeId::new(5, [1, 4], [8, 9], [5, 6]).unwrap());

        let bytes = set.to_bytes().unwrap();
        let restored = unsafe { SpatialIdSet::from_bytes(&bytes).unwrap() };

        assert_eq!(sorted(&set), sorted(&restored));
        assert_eq!(set.count(), restored.count());
    }

    #[test]
    fn round_trip_empty() {
        let set = SpatialIdSet::new();
        let bytes = set.to_bytes().unwrap();
        let restored = unsafe { SpatialIdSet::from_bytes(&bytes).unwrap() };
        assert!(restored.is_empty());
    }
}

#[derive(Default, Clone, Debug)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct SpatialIdSet {
    inner: FlexTreeCore<()>,
}

impl PartialEq for SpatialIdSet {
    fn eq(&self, other: &Self) -> bool {
        let common_z = self
            .max_zoomlevel()
            .unwrap_or(0)
            .max(other.max_zoomlevel().unwrap_or(0));

        self.normalized_single_ids_at_zoom(common_z)
            == other.normalized_single_ids_at_zoom(common_z)
    }
}

impl Eq for SpatialIdSet {}

impl SpatialIdSet {
    pub fn new() -> Self {
        SpatialIdSet::default()
    }

    pub fn insert<S: IterFlexIds>(&mut self, target: S) {
        self.inner.insert(target, ());
    }

    pub fn get<'a, S>(&'a self, target: &'a S) -> impl Iterator<Item = FlexId> + 'a
    where
        S: IterFlexIds,
    {
        self.inner.get(target).map(move |(flex_id, _value)| flex_id)
    }

    pub fn remove<S: IterFlexIds>(&mut self, target: &S) -> impl Iterator<Item = FlexId> {
        self.inner
            .remove(target)
            .map(move |(flex_id, _value)| flex_id)
    }

    /// [`get`](Self::get) と異なり切り取りを行わず、target と重なった
    /// [`FlexId`] をそのままの返します。
    pub fn get_overlapping<'a, S>(&'a self, target: &'a S) -> impl Iterator<Item = FlexId> + 'a
    where
        S: IterFlexIds + 'a,
    {
        self.inner
            .get_overlapping(target)
            .map(|(flex_id, _value)| flex_id)
    }

    /// [`get`](Self::get) と異なり切り取りを行わず、target と重なった
    /// [`FlexId`] をそのままの返します。
    pub fn remove_overlapping<S: IterFlexIds>(
        &mut self,
        target: &S,
    ) -> impl Iterator<Item = FlexId> {
        self.inner
            .remove_overlapping(target)
            .map(move |(flex_id, _value)| flex_id)
    }

    /// 指定した単体の空間 IDと面で接している[`FlexId`] を重複なく返します。入力された空間ID自身と重なる要素は除外します。
    pub fn neighbors_share_face<S: SpatialId>(
        &self,
        target: &S,
    ) -> impl Iterator<Item = FlexId> + '_ {
        self.inner
            .neighbors_share_face_ref(target)
            .map(|(flex_id, _value)| flex_id)
    }

    pub fn count(&self) -> usize {
        self.inner.count()
    }

    pub fn max_zoomlevel(&self) -> Option<u8> {
        self.inner.max_zoomlevel()
    }

    pub fn flat_single_ids(&self) -> impl Iterator<Item = SingleId> {
        self.inner
            .flat_single_ids_ref()
            .map(|(single_id, _)| single_id)
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    #[cfg(test)]
    pub fn root_ptr_eq(&self, other: &Self) -> bool {
        self.inner.root_ptr_eq(&other.inner)
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = FlexId> {
        self.inner.iter().map(|(flex_id, _)| flex_id)
    }

    /// この [`SpatialIdSet`] を rkyv バイト列へ直列化する（`feature = "persist"`）。
    #[cfg(feature = "persist")]
    pub fn to_bytes(&self) -> Result<alloc::vec::Vec<u8>, rkyv::rancor::Error> {
        Ok(rkyv::to_bytes::<rkyv::rancor::Error>(self)?.to_vec())
    }

    /// [`to_bytes`](Self::to_bytes) で直列化したバイト列から復元する（`feature = "persist"`）。
    ///
    /// # Safety
    /// `bytes` は [`SpatialIdSet::to_bytes`] が生成した正当なバイト列でなければならない。
    #[cfg(feature = "persist")]
    pub unsafe fn from_bytes(bytes: &[u8]) -> Result<Self, rkyv::rancor::Error> {
        let archived = unsafe { rkyv::access_unchecked::<ArchivedSpatialIdSet>(bytes) };
        rkyv::deserialize::<Self, rkyv::rancor::Error>(archived)
    }

    fn normalized_single_ids_at_zoom(&self, target_z: u8) -> HashSet<SingleId> {
        let mut normalized = HashSet::new();

        for flex_id in self.iter() {
            let range = RangeId::from(&flex_id);
            let expanded = if range.z() == target_z {
                range
            } else {
                range
                    .spatial_children_at_zoom(target_z)
                    .expect("target_z must be >= range.z")
            };

            for single_id in expanded.into_single_ids() {
                normalized.insert(single_id);
            }
        }

        normalized
    }
}
