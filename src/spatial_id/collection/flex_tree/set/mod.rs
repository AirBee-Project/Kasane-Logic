use hashbrown::HashSet;

use crate::{FlexId, FlexTreeCore, IntoSingleIds, IterFlexIds, RangeId, SingleId, SpatialId};
pub mod convert;
pub mod json;
pub mod ops;
pub mod tests;

/// 空間IDの集合を表す型。
///
/// `SpatialIdSet` は、保持する値が空間IDそのものだけであるため、「どの空間が存在するか」を表すための型として機能する。
///
/// - ある場所に対する空間IDを「存在しない」もしくは「一意に定まる」状態を維持する
/// - 集合同士の演算や、集合に対する単項演算を提供する
///
/// # 注意
/// - 現在は時空間IDに非対応で、時間ID部分がWHOLEではないIDが挿入された場合に無条件にPanicする。(将来的に時間IDにも対応する予定。)
/// - 空間ごとに値を持たせたい、値から空間を引きたい、または値の管理が必要な場合は [`SpatialIdTable`](crate::SpatialIdTable) を使用する。
#[derive(Default, Clone, Debug)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct SpatialIdSet {
    inner: FlexTreeCore<()>,
}

impl PartialEq for SpatialIdSet {
    // Todo:等価検証が重いのでどうにかする
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
    /// 新しい集合を作成する。
    ///
    /// # Examples
    ///
    /// ```
    /// use kasane_logic::SpatialIdSet;
    ///
    /// let set = SpatialIdSet::new();
    /// assert!(set.is_empty());
    /// ```
    pub fn new() -> Self {
        SpatialIdSet::default()
    }

    /// 限定的な領域に閉じた空の[SpatialIdSet]を作成する。
    /// `region` の内側だけを保持し、`region` の外側への操作は無視される。
    pub fn new_in_shard(region: FlexId) -> Self {
        Self {
            inner: FlexTreeCore::new_in_shard(region),
        }
    }

    /// 集合に対して空間IDを挿入する。[SpatialId] Traitが実装されていれば挿入ができる。
    /// 挿入した際に重なりがある空間IDが既に存在する場合は自動的に重なりを解消する。
    ///
    /// # Examples
    ///
    /// ```
    /// use kasane_logic::{FlexId, RangeId, SingleId, SpatialIdSet};
    ///
    /// let mut set = SpatialIdSet::new();
    ///
    /// // SingleId の挿入
    /// let single = SingleId::new(23, 0, 7451089, 3303245).unwrap();
    /// set.insert(single);
    ///
    /// // RangeId の挿入
    /// let range = RangeId::new(23, [0, 0], [7451089, 7451089], [3303245, 3303245]).unwrap();
    /// set.insert(range);
    ///
    /// // FlexId の挿入
    /// let flex = FlexId::new(23, 0, 24, 7451089, 23, 3303245).unwrap();
    /// set.insert(flex);
    /// ```
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
