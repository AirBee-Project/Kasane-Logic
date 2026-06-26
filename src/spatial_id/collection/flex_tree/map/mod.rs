use alloc::vec::Vec;

use crate::{FlexId, FlexTreeCore, IterFlexIds, SingleId, SpatialId};

pub mod convert;
pub mod json;

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

    /// マップに保持されている値への参照を重複なく返します。
    ///
    /// 値インデックスを持たないため `inner` を走査し、`PartialEq` で重複を除去します。
    pub fn values(&self) -> impl Iterator<Item = &V> + '_ {
        let mut out: Vec<&V> = Vec::new();
        for (_, value) in self.inner.iter_ref() {
            if !out.contains(&value) {
                out.push(value);
            }
        }
        out.into_iter()
    }
}

/// DB 用途（値＝バイト列）の永続化。ジェネリック境界を避けるため `Vec<u8>` 固定で提供する。
#[cfg(feature = "persist")]
impl SpatialIdMap<Vec<u8>> {
    /// この [`SpatialIdMap`] を rkyv バイト列へ直列化する。
    pub fn to_bytes(&self) -> Result<Vec<u8>, rkyv::rancor::Error> {
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

#[cfg(all(test, feature = "persist"))]
mod persist_tests {
    use super::SpatialIdMap;
    use crate::{RangeId, SingleId};
    use alloc::vec::Vec;

    fn sorted(map: &SpatialIdMap<Vec<u8>>) -> Vec<(crate::FlexId, Vec<u8>)> {
        let mut v: Vec<_> = map.iter().map(|(f, val)| (f, val.clone())).collect();
        v.sort();
        v
    }

    #[test]
    fn round_trip() {
        let mut map = SpatialIdMap::<Vec<u8>>::new();
        map.insert(SingleId::new(20, 0, 0, 0).unwrap(), b"alpha".to_vec());
        map.insert(SingleId::new(20, 0, 2, 3).unwrap(), b"alpha".to_vec());
        map.insert(SingleId::new(18, 1, 5, 7).unwrap(), b"beta".to_vec());
        map.insert(
            RangeId::new(5, [1, 4], [8, 9], [5, 6]).unwrap(),
            b"gamma".to_vec(),
        );

        let bytes = map.to_bytes().unwrap();
        let restored = unsafe { SpatialIdMap::<Vec<u8>>::from_bytes(&bytes).unwrap() };

        assert_eq!(sorted(&map), sorted(&restored));
        assert_eq!(map.count(), restored.count());
    }

    #[test]
    fn round_trip_empty() {
        let map = SpatialIdMap::<Vec<u8>>::new();
        let bytes = map.to_bytes().unwrap();
        let restored = unsafe { SpatialIdMap::<Vec<u8>>::from_bytes(&bytes).unwrap() };
        assert!(restored.is_empty());
    }
}
