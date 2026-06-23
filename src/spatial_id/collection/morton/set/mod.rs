use hashbrown::HashSet;

use crate::{IntoSingleIds, IterFlexIds, RangeId, SingleId, SpatialId};

use super::core::MortonCore;

pub mod convert;
pub mod json;
pub mod ops;

#[cfg(test)]
mod tests;

/// Morton order バックエンドの空間ID集合。
///
/// 公開 API は単一解像度の [`SingleId`] を返す（FlexTree 版が [`FlexId`](crate::FlexId) を返すのと
/// 異なる）。内部は [`MortonCore`] による単一解像度セルのフラットな順序付き集合。
#[derive(Default, Clone, Debug)]
pub struct SpatialIdSet {
    pub(crate) inner: MortonCore<()>,
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

    /// 対象と重なる [`SingleId`] を、重なり部分へクリップして返す。
    pub fn get<S>(&self, target: &S) -> impl Iterator<Item = SingleId>
    where
        S: IterFlexIds,
    {
        self.inner
            .get_single(target)
            .into_iter()
            .map(|(sid, _)| sid)
    }

    pub fn remove<S: IterFlexIds>(&mut self, target: &S) -> impl Iterator<Item = SingleId> {
        self.inner.remove(target).into_iter().map(|(sid, _)| sid)
    }

    /// 切り取りを行わず、target と重なった格納済み [`SingleId`] をそのまま返す。
    pub fn get_overlapping<S>(&self, target: &S) -> impl Iterator<Item = SingleId>
    where
        S: IterFlexIds,
    {
        self.inner
            .get_overlapping_single(target)
            .into_iter()
            .map(|(sid, _)| sid)
    }

    /// 切り取りを行わず、target と重なった格納済み [`SingleId`] を丸ごと取り除いて返す。
    pub fn remove_overlapping<S: IterFlexIds>(
        &mut self,
        target: &S,
    ) -> impl Iterator<Item = SingleId> {
        self.inner
            .remove_overlapping(target)
            .into_iter()
            .map(|(sid, _)| sid)
    }

    /// 入力した単体の空間IDと面で接している [`SingleId`] を重複なく返す。自身と重なる要素は除外する。
    pub fn neighbors_share_face<S: SpatialId>(&self, target: &S) -> impl Iterator<Item = SingleId> {
        self.inner
            .neighbors_share_face_single(target)
            .into_iter()
            .map(|(sid, _)| sid)
    }

    pub fn count(&self) -> usize {
        self.inner.count()
    }

    pub fn max_zoomlevel(&self) -> Option<u8> {
        self.inner.max_zoomlevel()
    }

    pub fn flat_single_ids(&self) -> impl Iterator<Item = SingleId> {
        self.inner.flat_single_ids().into_iter().map(|(sid, _)| sid)
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = SingleId> + '_ {
        self.inner.iter_single().map(|(sid, _)| sid)
    }

    fn normalized_single_ids_at_zoom(&self, target_z: u8) -> HashSet<SingleId> {
        let mut normalized = HashSet::new();

        for single in self.iter() {
            let range = RangeId::from(&single);
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
