use alloc::vec::Vec;
use hashbrown::HashSet;

use crate::spatial_id::collection::flex_tree::core::node_ops::{TSetDifference, TSetUnion};
use crate::{FlexId, FlexTreeCore, IntoSingleIds, RangeId, SingleId, SpatialId, TemporalSet};
pub mod convert;
pub mod impls;
pub mod json;
pub mod ops;
pub mod shard;
pub mod tests;

/// 時空間IDの集合を表す型。
///
/// `SpatialIdSet` は「どの空間が、どの時間に存在するか」を表すための型として機能する。
///
/// - 空間は木構造（FlexTree）の一次索引として、時間は各空間セルの値
///   （[`TemporalSet`]）として保持する（**時間ネイティブ**）。
/// - 時間IDが全時間（WHOLE）のIDだけを扱う場合は、従来どおり純粋な空間集合として振る舞う。
/// - 集合同士の演算（和・積・差）は空間×時間の4次元で厳密に行われる。
///
/// # 注意
/// - 空間ごとに値を持たせたい、値から空間を引きたい、または値の管理が必要な場合は [`SpatialIdTable`](crate::SpatialIdTable) を使用する。
#[derive(Default, Clone, Debug)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct SpatialIdSet {
    inner: FlexTreeCore<TemporalSet>,
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
    pub fn insert<S: SpatialId>(&mut self, target: S) {
        for flex_id in target.iter_flex_ids() {
            // シャード領域外は無視し、はみ出しは切り詰める。
            let flex_id = match self.inner.shard() {
                Some(region) => match flex_id.intersection(region) {
                    Some(clipped) => clipped,
                    None => continue,
                },
                None => flex_id,
            };
            let temporal = flex_id.temporal().clone();
            let spatial = flex_id.spatial_part();
            if temporal.is_whole() {
                // 全時間はどんな既存時間も吸収する（whole ∪ X = whole）ので、
                // 覆う領域を置換する直接挿入がそのまま union になる。
                self.inner.insert_flex_id(spatial, TemporalSet::whole());
            } else {
                // 既存と空間が重なる領域では時間を union して合成する。
                let mut single = FlexTreeCore::<TemporalSet>::new();
                single.insert_flex_id(spatial, TemporalSet::from_temporal(&temporal));
                let shard = self.inner.shard().cloned();
                self.inner = self.inner.combine_with::<TSetUnion>(&single, shard);
            }
        }
    }

    /// 集合から指定した時空間IDと重なる時空間IDを切り出して返す。
    ///
    /// 空間・時間の両方が query に切り取られる（時間は query の時間との交差）。
    pub fn get<'a, S>(&'a self, target: &'a S) -> impl Iterator<Item = FlexId> + 'a
    where
        S: SpatialId,
    {
        self.inner.get_ref(target).flat_map(|(clipped, tset)| {
            // clipped の時間は query 由来。実際の存在時間（値）との交差を付けて返す。
            let overlap = tset.intersection(&TemporalSet::from_temporal(clipped.temporal()));
            overlap
                .cells()
                .into_iter()
                .map(|t| clipped.with_temporal(t))
                .collect::<Vec<_>>()
        })
    }
    /// 集合から指定した時空間IDと重なる部分を切り出して削除する。
    /// 削除した部分の時空間IDを返す。
    pub fn remove<S: SpatialId>(&mut self, target: &S) -> impl Iterator<Item = FlexId> {
        let removed: Vec<FlexId> = self.get(target).collect();
        let mut query = SpatialIdSet::new();
        for flex_id in target.iter_flex_ids() {
            query.insert(flex_id);
        }
        let shard = self.inner.shard().cloned();
        self.inner = self
            .inner
            .combine_with::<TSetDifference>(&query.inner, shard);
        removed.into_iter()
    }

    /// 指定した空間IDと接触していたすべての空間IDを返す。
    /// [`get`](Self::get) と異なり切り取りを行わず、target と重なった [`FlexId`] をそのままの返す。
    pub fn get_overlapping<'a, S>(&'a self, target: &'a S) -> impl Iterator<Item = FlexId> + 'a
    where
        S: SpatialId + 'a,
    {
        self.inner
            .get_overlapping_ref(target)
            .flat_map(|(stored, tset)| {
                tset.cells()
                    .into_iter()
                    .map(|t| stored.with_temporal(t))
                    .collect::<Vec<_>>()
            })
    }

    /// 指定した空間IDと接触していたすべての空間IDを削除する。削除した時空間IDを返す。
    /// [`remove`](Self::remove) と異なり切り取りを行わず、target と空間的に重なった葉を
    /// （その全時間ごと）丸ごと取り除く。
    pub fn remove_overlapping<S: SpatialId>(&mut self, target: &S) -> impl Iterator<Item = FlexId> {
        let mut removed = Vec::new();
        for (stored, tset) in self.inner.remove_overlapping(target) {
            for t in tset.cells() {
                removed.push(stored.with_temporal(t));
            }
        }
        removed.into_iter()
    }

    /// 指定した単体の空間 IDと面で接している[`FlexId`] を重複なく返す。入力された空間ID自身と重なる空間IDは除外する。
    /// 面共有の判定は空間3軸のみで行う（時間は考慮しない）。返る [`FlexId`] には存在時間が付く。
    pub fn neighbors_share_face<S: SpatialId>(
        &self,
        target: &S,
    ) -> impl Iterator<Item = FlexId> + '_ {
        self.inner
            .neighbors_share_face_ref(target)
            .flat_map(|(stored, tset)| {
                tset.cells()
                    .into_iter()
                    .map(|t| stored.with_temporal(t))
                    .collect::<Vec<_>>()
            })
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

    /// [SpatialIdSet]の最大のズームレベル値に揃えて、すべてを `SingleId` として返す。
    /// 各 [`SingleId`] には存在時間（時間セル）が付く。
    pub fn flat_single_ids(&self) -> impl Iterator<Item = SingleId> {
        let Some(max_zoomlevel) = self.max_zoomlevel() else {
            return Vec::new().into_iter();
        };

        let mut exported = Vec::new();
        for flex_id in self.iter() {
            let range = RangeId::from(&flex_id);
            let normalized = if range.z() == max_zoomlevel {
                range
            } else {
                range
                    .spatial_children_at_zoom(max_zoomlevel)
                    .expect("target max zoomlevel must be valid")
            };
            exported.extend(normalized.into_single_ids());
        }
        exported.into_iter()
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

    /// 保持している全ての時空間IDを返す。
    ///
    /// 各空間セルの存在時間（[`TemporalSet`]）は約数鎖の最小セル列へ分解され、
    /// `(空間セル × 時間セル)` の [`FlexId`] として列挙される。
    /// 全時間（WHOLE）のセルは従来どおり1つの [`FlexId`]（temporal=WHOLE）になる。
    pub fn iter(&self) -> impl Iterator<Item = FlexId> + '_ {
        self.inner.iter_ref().flat_map(|(spatial, tset)| {
            tset.cells()
                .into_iter()
                .map(|t| spatial.with_temporal(t))
                .collect::<Vec<_>>()
        })
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
