use fxhash::FxBuildHasher;
use hashbrown::HashSet;

use crate::{FlexId, FlexTreeCore, IntoSingleIds, IterFlexIds, RangeId, SingleId, SpatialId};
pub mod convert;
pub mod json;
pub mod ops;
pub mod tests;

#[derive(Default, Clone, Debug)]
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

    fn normalized_single_ids_at_zoom(&self, target_z: u8) -> HashSet<SingleId, FxBuildHasher> {
        let mut normalized: HashSet<SingleId, FxBuildHasher> = HashSet::default();

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
