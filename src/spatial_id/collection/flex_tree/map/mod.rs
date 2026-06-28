use crate::{FlexId, FlexTreeCore, IterFlexIds, SingleId, SpatialId};

pub mod convert;
pub mod json;
pub mod tests;

/// 空間(FlexId)に値(V)を対応づけるマップ構造。
///
/// [`SpatialIdTable`](crate::SpatialIdTable) と基本 API は同じだが、
/// 値→空間の逆引き（値インデックス）や値↔ランク辞書を一切持たず、
/// 値を直接ツリーに格納する index-free 版。そのため `V: Ord` を要求せず、
/// `value_get` / `value_range` / `rebuild_index` のような値クエリは提供しない。
#[derive(Default, Clone, Debug)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "persist", rkyv(archive_bounds(V: 'static)))]
#[cfg_attr(
    feature = "persist",
    rkyv(serialize_bounds(
        __S: rkyv::ser::Writer + rkyv::ser::Allocator + rkyv::ser::Sharing,
        <__S as rkyv::rancor::Fallible>::Error: rkyv::rancor::Source,
    ))
)]
#[cfg_attr(
    feature = "persist",
    rkyv(deserialize_bounds(
        __D: rkyv::de::Pooling,
        <__D as rkyv::rancor::Fallible>::Error: rkyv::rancor::Source,
    ))
)]
#[cfg_attr(
    feature = "persist",
    rkyv(bytecheck(bounds(
        __C: rkyv::validation::ArchiveContext + rkyv::validation::SharedContext,
        <__C as rkyv::rancor::Fallible>::Error: rkyv::rancor::Source,
    )))
)]
pub struct SpatialIdMap<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue,
{
    inner: FlexTreeCore<V>,
}

impl<V> SpatialIdMap<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue,
{
    /// 空の [`SpatialIdMap`] を作成します。
    pub fn new() -> Self {
        Self {
            inner: FlexTreeCore::default(),
        }
    }

    /// シャード領域 `region` に閉じた空の[SpatialIdMap]を作成する。
    ///
    /// 以降は `region` の内側だけを保持する。`region` の外側への挿入は無視される。
    pub fn new_in_shard(region: FlexId) -> Self {
        Self {
            inner: FlexTreeCore::new_in_shard(region),
        }
    }

    /// 空間に値を挿入します。
    pub fn insert<S: IterFlexIds>(&mut self, target: S, value: V) {
        self.inner.insert(target, value);
    }

    /// 特定の空間（target）と交差するすべての領域と、その値への参照を返します。
    pub fn get<'a, S>(&'a self, target: &'a S) -> impl Iterator<Item = (FlexId, &'a V)> + 'a
    where
        S: IterFlexIds,
    {
        self.inner.get_ref(target)
    }

    /// 指定した空間（target）をツリーからくり抜き、削除された領域とその値を返します。
    pub fn remove<S: IterFlexIds>(&mut self, target: &S) -> impl Iterator<Item = (FlexId, V)> {
        self.inner.remove(target)
    }

    /// [`get`](Self::get) と異なり切り取りを行わず、target と重なった
    /// [`FlexId`]と値への参照をそのまま返します。
    pub fn get_overlapping<'a, S>(
        &'a self,
        target: &'a S,
    ) -> impl Iterator<Item = (FlexId, &'a V)> + 'a
    where
        S: IterFlexIds + 'a,
    {
        self.inner.get_overlapping_ref(target)
    }

    /// [`remove`](Self::remove) と異なり切り取りを行わず、target と重なった
    /// [`FlexId`]と値をそのまま取り除いて返します。
    pub fn remove_overlapping<S: IterFlexIds>(
        &mut self,
        target: &S,
    ) -> impl Iterator<Item = (FlexId, V)> {
        self.inner.remove_overlapping(target)
    }

    /// 指定した単体の空間 IDと面で接している[`FlexId`]と値への参照を重複なく返します。
    /// 入力された空間ID自身と重なる要素は除外します。
    pub fn neighbors_share_face<'a, S: SpatialId>(
        &'a self,
        target: &S,
    ) -> impl Iterator<Item = (FlexId, &'a V)> + 'a {
        self.inner.neighbors_share_face_ref(target)
    }

    /// 保持している[FlexId]の総数を返します。
    pub fn count(&self) -> usize {
        self.inner.count()
    }

    /// このマップをシャード分割すべきか判定する。**O(1)**。
    /// 保持する FlexId 数が `max_flex_id_count` を超えていれば `true`。
    pub fn should_split_shard(&self, max_flex_id_count: usize) -> bool {
        self.inner.should_split_shard(max_flex_id_count)
    }

    /// 最も均衡する位置で2つのシャードへ二分割する。**O(Z)**。
    /// 分割した場合は `Some((A, B))`、FlexId が1つ以下で分割できない場合は `None`。
    pub fn split_shard(&self) -> Option<(Self, Self)> {
        self.inner
            .split_shard()
            .map(|(a, b)| (Self { inner: a }, Self { inner: b }))
    }

    /// ツリーの最大ズームレベルを返します。
    pub fn max_zoomlevel(&self) -> Option<u8> {
        self.inner.max_zoomlevel()
    }

    /// 最下層の[SingleId]レベルまで展開したイテレータを参照付きで返します。
    pub fn flat_single_ids(&self) -> impl Iterator<Item = (SingleId, &V)> + '_ {
        self.inner.flat_single_ids_ref()
    }

    /// マップが空かどうかを返します。
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// 全てをクリアします。
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// マップに保持されている全ての空間と値への参照のペアを返します。
    pub fn iter(&self) -> impl Iterator<Item = (FlexId, &V)> + '_ {
        self.inner.iter_ref()
    }
}

/// DB 用途（値＝バイト列）の永続化。ジェネリック境界を避けるため `Vec<u8>` 固定で提供する。
#[cfg(feature = "persist")]
impl SpatialIdMap<Vec<u8>> {
    /// この [`SpatialIdMap`] を rkyv バイト列へ直列化する。
    pub fn to_bytes(&self) -> Result<alloc::vec::Vec<u8>, rkyv::rancor::Error> {
        Ok(rkyv::to_bytes::<rkyv::rancor::Error>(self)?.to_vec())
    }

    /// [`to_bytes`](Self::to_bytes) で直列化したバイト列から復元する。
    ///
    /// # Safety
    /// `bytes` は [`SpatialIdMap::to_bytes`] が生成した正当なバイト列でなければならない。
    pub unsafe fn from_bytes(bytes: &[u8]) -> Result<Self, rkyv::rancor::Error> {
        let archived = unsafe { rkyv::access_unchecked::<ArchivedSpatialIdMap<Vec<u8>>>(bytes) };
        rkyv::deserialize::<Self, rkyv::rancor::Error>(archived)
    }
}
