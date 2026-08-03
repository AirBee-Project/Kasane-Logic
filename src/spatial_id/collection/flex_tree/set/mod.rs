use crate::spatial_id::collection::flex_tree::core::FlexTreeCore;
use crate::{AllowedIntervals, FlexId, RangeId, SingleId, SpatialId};
use alloc::vec::Vec;
pub mod convert;
pub mod impls;
#[cfg(feature = "json")]
pub mod json;
pub mod ops;
pub mod shard;
pub mod tests;

/// 空間IDの集合を表す型。
///
/// `SpatialIdSet` は、保持する値が空間IDそのものだけであるため、「どの空間が存在するか」を表すための型として機能する。
///
/// - ある場所に対する空間IDを「存在しない」もしくは「一意に定まる」状態を維持する
/// - 集合同士の演算や、集合に対する単項演算を提供する
///
/// # 使い分け
/// - 空間ごとに値を持たせたい場合は [`SpatialIdMap`](crate::SpatialIdMap) を使用する。
/// - 値から空間を引きたい、または値の管理（重複排除など）が必要な場合は
///   [`SpatialIdTable`](crate::SpatialIdTable) を使用する。
#[derive(Default, Clone, Debug)]
pub struct SpatialIdSet {
    inner: FlexTreeCore<()>,
}

impl PartialEq for SpatialIdSet {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
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

    /// 内部 [`FlexTreeCore`] から集合を組む（クエリ実行の出口変換用）。
    pub(crate) fn from_core(inner: FlexTreeCore<()>) -> Self {
        Self { inner }
    }

    /// この集合が値を持つ全Segmentを包む最小の[RangeId]を返します。
    pub fn bounding_box(&self) -> Option<RangeId> {
        self.inner.bounding_box()
    }

    /// 所有権ごと内部 [`FlexTreeCore`] を取り出す（クエリ実行の入口変換用）。
    pub(crate) fn into_core(self) -> FlexTreeCore<()> {
        self.inner
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
    pub fn insert<S: SpatialId>(&mut self, target: S) {
        self.inner.insert(target, ());
    }

    /// 集合から指定した空間IDと重なる空間IDを切り出して返す。
    pub fn get<'a, S>(&'a self, target: &'a S) -> impl Iterator<Item = FlexId> + 'a
    where
        S: SpatialId,
    {
        self.inner
            .get(target.clone())
            .map(move |(flex_id, _value)| flex_id)
    }

    /// 指定した範囲（RangeId）と重なる空間IDを切り出して返す。
    pub fn get_range<'a>(
        &'a self,
        target: &'a crate::RangeId,
    ) -> impl Iterator<Item = FlexId> + 'a {
        self.inner
            .range_overlap_ref(target)
            .map(|(flex_id, _value)| flex_id)
    }

    /// 集合から指定した空間IDと重なる空間IDを切り出して削除する。
    /// 削除した部分の空間IDを返す。
    pub fn remove<S: SpatialId>(&mut self, target: &S) -> Vec<FlexId> {
        self.inner
            .remove(target.clone())
            .into_iter()
            .map(|(flex_id, _value)| flex_id)
            .collect()
    }

    /// 指定した空間IDと接触していたすべての空間IDを返す。
    /// [`get`](Self::get) と異なり切り取りを行わず、target と重なった [`FlexId`] をそのままの返す。
    pub fn get_overlapping<'a, S>(&'a self, target: &'a S) -> impl Iterator<Item = FlexId> + 'a
    where
        S: SpatialId + 'a,
    {
        self.inner
            .get_overlapping(target.clone())
            .map(|(flex_id, _value)| flex_id)
    }

    /// 指定した空間IDと接触していたすべての空間IDを削除する。削除した空間IDを返す。
    /// [`remove`](Self::remove) と異なり切り取りを行わず、target と重なった [`FlexId`] をそのまま返す。
    pub fn remove_overlapping<S: SpatialId>(&mut self, target: &S) -> Vec<FlexId> {
        self.inner
            .remove_overlapping(target.clone())
            .into_iter()
            .map(|(flex_id, _value)| flex_id)
            .collect()
    }

    /// 指定した単体の空間 IDと面で接している[`FlexId`] を重複なく返す。入力された空間ID自身と重なる空間IDは除外する。
    pub fn neighbors_share_face<S: SpatialId>(
        &self,
        target: &S,
    ) -> impl Iterator<Item = FlexId> + '_ {
        self.inner
            .neighbors_share_face_ref(target)
            .map(|(flex_id, _value)| flex_id)
    }

    /// 集合の内部にある[FlexId]の個数を返す。
    pub fn count(&self) -> usize {
        self.inner.count()
    }

    /// 集合の内部にある全ての[FlexId]のうち、最大のズームレベル値を返す。
    /// 内部に空間IDが存在しない場合は[None]を返します。
    pub fn max_zoomlevel(&self) -> Option<u8> {
        self.inner.max_zoomlevel()
    }

    /// 時間方向に結合した [`RangeId`] として読み出す。**空間解像度は変えない**。
    ///
    /// 単位は「その区間を表せる最も粗い秒数」（`gcd(開始秒, 幅)`）が選ばれ、
    /// 各エントリのSegment数は必ず1になる。単位を選びたい場合は
    /// [`range_ids_in`](Self::range_ids_in) を使う。
    ///
    /// [`iter`](Self::iter) が返す生の [`FlexId`] は木の2分岐Segmentそのもの
    /// （`_8/182185424` のような断片）なので、人間が読む用途にはこちらを使う。
    pub fn range_ids(&self) -> impl Iterator<Item = RangeId> + '_ {
        self.inner.range_ids_ref(None).map(|(range_id, _)| range_id)
    }

    /// 時間の単位を [`AllowedIntervals`] の候補から選んで読み出す。
    ///
    /// 候補のうち**その区間を割り切る最も粗いもの**が選ばれる（＝候補の中でSegment数が最小）。
    /// [`AllowedIntervals`] は必ず全区間を表せる候補を含むので失敗しない。
    ///
    /// ```
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::{Interval, AllowedIntervals, SingleId, SpatialIdSet};
    /// let mut set = SpatialIdSet::new();
    /// set.insert(SingleId::new(12, 0, 3638, 1614).unwrap().with_time(Interval::HOUR, 0).unwrap());
    /// set.insert(SingleId::new(12, 0, 3638, 1614).unwrap().with_time(Interval::HOUR, 1).unwrap());
    ///
    /// // 既定（gcd）では 2 時間ぶんが「7200 秒 × 1 Segment」になる。
    /// assert_eq!(set.range_ids().next().unwrap().to_string(), "12/0/3638/1614_7200/0");
    ///
    /// // 暦の単位に正規化すると「3600 秒 × 2 Segment」になる。
    /// let got = set.range_ids_in(AllowedIntervals::calendar()).next().unwrap();
    /// assert_eq!(got.to_string(), "12/0/3638/1614_3600/0:1");
    /// # }
    /// ```
    pub fn range_ids_in<'a>(
        &'a self,
        units: &'a AllowedIntervals,
    ) -> impl Iterator<Item = RangeId> + use<'a> {
        self.inner
            .range_ids_ref(Some(units))
            .map(|(range_id, _)| range_id)
    }

    /// [`flat_single_ids`](Self::flat_single_ids) の、時間単位を指定できる版。
    pub fn flat_single_ids_in<'a>(
        &'a self,
        units: &'a AllowedIntervals,
    ) -> impl Iterator<Item = SingleId> + use<'a> {
        self.inner
            .flat_single_ids_in_ref(Some(units))
            .map(|(single_id, _)| single_id)
    }

    /// [SpatialIdSet]の最大のズームレベル値に揃えて、すべてを `SingleId` として返す。
    pub fn flat_single_ids(&self) -> impl Iterator<Item = SingleId> {
        self.inner
            .flat_single_ids_ref()
            .map(|(single_id, _)| single_id)
    }

    /// [SpatialIdSet]の内部の空間IDを全て削除します。
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    #[cfg(test)]
    pub fn root_ptr_eq(&self, other: &Self) -> bool {
        self.inner.root_ptr_eq(&other.inner)
    }

    /// [SpatialIdSet]の内部が空かどうかを判定します。
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = FlexId> {
        self.inner.iter().map(|(flex_id, _)| flex_id)
    }
}

pub struct SpatialIdSetIntoIter {
    inner: crate::spatial_id::collection::flex_tree::core::LeavesIntoIter<()>,
}

impl Iterator for SpatialIdSetIntoIter {
    type Item = (FlexId, ());

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(flex_id, _)| (flex_id, ()))
    }
}

impl IntoIterator for SpatialIdSet {
    type Item = (FlexId, ());
    type IntoIter = SpatialIdSetIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        SpatialIdSetIntoIter {
            inner: self.inner.into_iter(),
        }
    }
}
