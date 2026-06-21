use embed_doc_image::embed_doc_image;
use hashbrown::HashSet;

use crate::{FlexId, FlexTreeCore, IntoSingleIds, RangeId, SingleId, SpatialId};
pub mod convert;
pub mod impls;
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
/// - 空間ごとに値を持たせたい、値から空間を引きたい、または値の一覧管理が必要な場合は [`SpatialIdTable`](crate::SpatialIdTable) を使用する。

#[derive(Default, Clone, Debug)]
pub struct SpatialIdSet {
    inner: FlexTreeCore<()>,
}

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
    /// let flex = FlexId::new(23, 0, 23, 7451089, 23, 3303245).unwrap();
    /// set.insert(flex);
    ///
    /// // 競合が解消される
    /// assert_eq!(set.count(), 1);
    /// ```
    pub fn insert<S: SpatialId>(&mut self, target: S) {
        self.inner.insert(target, ());
    }

    /// 集合から指定した空間IDと重なる空間IDを切り出して返す。
    /// ![画像の代替テキスト(Alt)][foobaring]
    /// # Examples
    ///
    /// ```
    /// use kasane_logic::{RangeId, SpatialIdSet};
    ///
    /// let range_id = RangeId::new(23, [0, 3], [7451089, 7451093], [3303245, 3303250]).unwrap();
    ///
    /// ```
    #[embed_doc_image("foobaring", "assets/image.png")]
    pub fn get<'a, S>(&'a self, target: &'a S) -> impl Iterator<Item = FlexId> + 'a
    where
        S: SpatialId,
    {
        self.inner.get(target).map(move |(flex_id, _value)| flex_id)
    }

    /// 集合から指定した空間IDと重なる空間IDを切り出して削除する。
    /// 削除した部分の空間IDを返す。
    ///
    /// # Examples
    ///
    /// ```
    /// use kasane_logic::{SingleId, SpatialIdSet};
    ///
    /// let mut set = SpatialIdSet::new();
    /// let id = SingleId::new(5, 0, 10, 10).unwrap();
    /// set.insert(id.clone());
    ///
    /// let removed: Vec<_> = set.remove(&id).collect();
    /// assert_eq!(removed.len(), 1);
    /// assert!(set.is_empty());
    /// ```
    pub fn remove<S: SpatialId>(&mut self, target: &S) -> impl Iterator<Item = FlexId> {
        self.inner
            .remove(target)
            .map(move |(flex_id, _value)| flex_id)
    }

    /// [`get`](Self::get) と異なり切り取りを行わず、`target` と重なった
    /// [`FlexId`] をそのまま返す。
    ///
    /// # Examples
    ///
    /// ```
    /// use kasane_logic::{SingleId, SpatialIdSet};
    ///
    /// let mut set = SpatialIdSet::new();
    /// let id = SingleId::new(5, 0, 10, 10).unwrap();
    /// set.insert(id.clone());
    ///
    /// let overlapping: Vec<_> = set.get_overlapping(&id).collect();
    /// assert_eq!(overlapping.len(), 1);
    /// ```
    pub fn get_overlapping<'a, S>(&'a self, target: &'a S) -> impl Iterator<Item = FlexId> + 'a
    where
        S: SpatialId + 'a,
    {
        self.inner
            .get_overlapping(target)
            .map(|(flex_id, _value)| flex_id)
    }

    /// [`remove`](Self::remove) と異なり切り取りを行わず、`target` と重なった
    /// [`FlexId`] をそのまま削除する。削除した空間IDを返す。
    ///
    /// # Examples
    ///
    /// ```
    /// use kasane_logic::{SingleId, SpatialIdSet};
    ///
    /// let mut set = SpatialIdSet::new();
    /// let id = SingleId::new(5, 0, 10, 10).unwrap();
    /// set.insert(id.clone());
    ///
    /// let removed: Vec<_> = set.remove_overlapping(&id).collect();
    /// assert_eq!(removed.len(), 1);
    /// assert!(set.is_empty());
    /// ```
    pub fn remove_overlapping<S: SpatialId>(&mut self, target: &S) -> impl Iterator<Item = FlexId> {
        self.inner
            .remove_overlapping(target)
            .map(move |(flex_id, _value)| flex_id)
    }

    /// 指定した単体の空間IDと面で接している[`FlexId`]を重複なく返す。入力された空間ID自身と重なる要素は除外する。
    ///
    /// # Examples
    ///
    /// ```
    /// use kasane_logic::{SingleId, SpatialIdSet};
    ///
    /// let mut set = SpatialIdSet::new();
    /// let center = SingleId::new(5, 0, 10, 10).unwrap();
    /// let neighbor = SingleId::new(5, 0, 11, 10).unwrap();
    /// set.insert(neighbor);
    ///
    /// let neighbors: Vec<_> = set.neighbors_share_face(&center).collect();
    /// assert_eq!(neighbors.len(), 1);
    /// ```
    pub fn neighbors_share_face<S: SpatialId>(
        &self,
        target: &S,
    ) -> impl Iterator<Item = FlexId> + '_ {
        self.inner
            .neighbors_share_face_ref(target)
            .map(|(flex_id, _value)| flex_id)
    }

    /// `SpatialIdSet` の中に入っている `FlexId` の個数を返す。
    ///
    /// # Examples
    ///
    /// ```
    /// use kasane_logic::{SingleId, SpatialIdSet};
    ///
    /// let mut set = SpatialIdSet::new();
    /// assert_eq!(set.count(), 0);
    /// set.insert(SingleId::new(5, 0, 10, 10).unwrap());
    /// assert_eq!(set.count(), 1);
    /// ```
    pub fn count(&self) -> usize {
        self.inner.count()
    }

    /// `SpatialIdSet` の中に入っている最大のズームレベル値を返す。
    ///
    /// # Examples
    ///
    /// ```
    /// use kasane_logic::{SingleId, SpatialIdSet};
    ///
    /// let mut set = SpatialIdSet::new();
    /// assert_eq!(set.max_zoomlevel(), None);
    /// set.insert(SingleId::new(5, 0, 10, 10).unwrap());
    /// assert_eq!(set.max_zoomlevel(), Some(5));
    /// ```
    pub fn max_zoomlevel(&self) -> Option<u8> {
        self.inner.max_zoomlevel()
    }

    /// `SpatialIdSet` の最大のズームレベル値に揃えて、すべてを `SingleId` として返す。
    ///
    /// # Examples
    ///
    /// ```
    /// use kasane_logic::{SingleId, SpatialIdSet};
    ///
    /// let mut set = SpatialIdSet::new();
    /// set.insert(SingleId::new(5, 0, 10, 10).unwrap());
    /// let flat: Vec<_> = set.flat_single_ids().collect();
    /// assert_eq!(flat.len(), 1);
    /// ```
    pub fn flat_single_ids(&self) -> impl Iterator<Item = SingleId> {
        self.inner
            .flat_single_ids_ref()
            .map(|(single_id, _)| single_id)
    }

    /// `SpatialIdSet` をリセットする。
    ///
    /// # Examples
    ///
    /// ```
    /// use kasane_logic::{SingleId, SpatialIdSet};
    ///
    /// let mut set = SpatialIdSet::new();
    /// set.insert(SingleId::new(5, 0, 10, 10).unwrap());
    /// set.clear();
    /// assert!(set.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// 内部ツリーのルートポインタが一致するかどうかを判定する。
    #[cfg(test)]
    pub(crate) fn root_ptr_eq(&self, other: &Self) -> bool {
        self.inner.root_ptr_eq(&other.inner)
    }

    /// `SpatialIdSet` が空かどうかを判定する。
    ///
    /// # Examples
    ///
    /// ```
    /// use kasane_logic::SpatialIdSet;
    ///
    /// let set = SpatialIdSet::new();
    /// assert!(set.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// `SpatialIdSet` が持つ `FlexId` を返す。
    ///
    /// # Examples
    ///
    /// ```
    /// use kasane_logic::{SingleId, SpatialIdSet};
    ///
    /// let mut set = SpatialIdSet::new();
    /// set.insert(SingleId::new(5, 0, 10, 10).unwrap());
    /// let items: Vec<_> = set.iter().collect();
    /// assert_eq!(items.len(), 1);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = FlexId> {
        self.inner.iter().map(|(flex_id, _)| flex_id)
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
