use alloc::vec::Vec;

use alloc::collections::{BTreeMap, BTreeSet};
use core::ops::RangeBounds;

pub mod impls;

pub mod tests;

#[cfg(all(test, feature = "persist"))]
mod persist_tests {
    use super::SpatialIdTable;
    use crate::{RangeId, SingleId};
    use alloc::vec::Vec;

    fn sorted(table: &SpatialIdTable<Vec<u8>>) -> Vec<(crate::FlexId, Vec<u8>)> {
        let mut v: Vec<_> = table.iter().map(|(f, val)| (f, val.clone())).collect();
        v.sort();
        v
    }

    #[test]
    fn round_trip() {
        let mut table = SpatialIdTable::<Vec<u8>>::new();
        table.insert(SingleId::new(20, 0, 0, 0).unwrap(), b"alpha".to_vec());
        table.insert(SingleId::new(20, 0, 2, 3).unwrap(), b"alpha".to_vec());
        table.insert(SingleId::new(18, 1, 5, 7).unwrap(), b"beta".to_vec());
        table.insert(
            RangeId::new(5, [1, 4], [8, 9], [5, 6]).unwrap(),
            b"gamma".to_vec(),
        );

        let bytes = table.to_bytes().unwrap();
        let restored = unsafe { SpatialIdTable::<Vec<u8>>::from_bytes(&bytes).unwrap() };

        assert_eq!(sorted(&table), sorted(&restored));
        assert_eq!(table.count(), restored.count());
    }

    #[test]
    fn round_trip_empty() {
        let table = SpatialIdTable::<Vec<u8>>::new();
        let bytes = table.to_bytes().unwrap();
        let restored = unsafe { SpatialIdTable::<Vec<u8>>::from_bytes(&bytes).unwrap() };
        assert!(restored.is_empty());
    }

    #[test]
    fn restored_is_mutable() {
        let mut table = SpatialIdTable::<Vec<u8>>::new();
        table.insert(SingleId::new(20, 0, 0, 0).unwrap(), b"alpha".to_vec());
        let bytes = table.to_bytes().unwrap();
        let mut restored = unsafe { SpatialIdTable::<Vec<u8>>::from_bytes(&bytes).unwrap() };
        let before = restored.count();
        restored.insert(SingleId::new(20, 0, 100, 100).unwrap(), b"delta".to_vec());
        assert_eq!(restored.count(), before + 1);
    }
}

use crate::spatial_id::collection::temporal::SpatioTemporalCore;
use crate::{FlexId, RangeId, SingleId, SpatialId, SpatialIdSet};

/// 値(V)と時空間(FlexId)を相互に高速検索・管理するための型。
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
pub struct SpatialIdTable<V>
where
    V: crate::spatial_id::collection::flex_tree::ptr::SafeValue + Ord,
{
    // メインの空間ツリー (空間 -> 時間ごとの Rank)
    inner: SpatioTemporalCore<usize>,

    // 辞書 (値 -> Rank)
    dictionary: BTreeMap<V, usize>,

    // 逆引き辞書 (Rank -> 値)
    reverse_dictionary: BTreeMap<usize, V>,

    // 逆引きインデックス (Rank -> その値が存在する空間の集合)
    //
    // 値クエリは未構築なら `inner` 走査で答える。明示的に [`rebuild_index`](Self::rebuild_index)を呼んだときだけ構築され、`value_index_built` が true になる。
    value_index: BTreeMap<usize, SpatialIdSet>,

    // `value_index` が `inner` と整合しているか（= 値クエリで使ってよいか）。
    value_index_built: bool,

    // 次に発行する一意なID（Rank）
    current_rank: usize,
}

impl<V> SpatialIdTable<V>
where
    V: crate::spatial_id::collection::flex_tree::ptr::SafeValue + Ord,
{
    /// 空の[SpatialIdTable]を作成します。
    pub fn new() -> Self {
        Self {
            inner: SpatioTemporalCore::new(),
            dictionary: BTreeMap::default(),
            reverse_dictionary: BTreeMap::default(),
            value_index: BTreeMap::default(),
            value_index_built: true,
            current_rank: 0,
        }
    }

    /// 時空間に値を挿入する。
    pub fn insert<S: SpatialId + Clone>(&mut self, target: S, value: V) {
        let rank = match self.dictionary.get(&value) {
            Some(v) => *v,
            None => {
                self.current_rank += 1;
                self.reverse_dictionary
                    .insert(self.current_rank, value.clone());
                self.dictionary.insert(value, self.current_rank);
                self.current_rank
            }
        };

        for flex_id in target.iter_flex_ids() {
            self.inner.insert_flex_id(flex_id, rank);
        }
        self.value_index_built = false;
    }

    /// 特定の時空間（target）と交差するすべての領域と、その値への参照を返す。
    pub fn get<'a, S>(&'a self, target: &'a S) -> impl Iterator<Item = (FlexId, &'a V)> + 'a
    where
        S: SpatialId,
    {
        self.inner.get(target).map(|(flex_id, rank)| {
            let value = self.reverse_dictionary.get(rank).unwrap();
            (flex_id, value)
        })
    }

    /// 指定した時空間（target）をツリーからくり抜き、削除された領域とその値を返す。
    pub fn remove<S: SpatialId + Clone>(
        &mut self,
        target: &S,
    ) -> impl Iterator<Item = (FlexId, V)> {
        let removed: Vec<(FlexId, usize)> = self.inner.remove(target).collect();
        let results: Vec<(FlexId, V)> = removed
            .into_iter()
            .map(|(flex_id, rank)| {
                let value = self.reverse_dictionary.get(&rank).unwrap().clone();
                (flex_id, value)
            })
            .collect();
        if !results.is_empty() {
            self.value_index_built = false;
        }
        results.into_iter()
    }
    /// [`get`](Self::get) と異なり切り取りを行わず、target と重なった[`FlexId`]と値をそのままの返す。
    pub fn get_overlapping<'a, S>(
        &'a self,
        target: &'a S,
    ) -> impl Iterator<Item = (FlexId, &'a V)> + 'a
    where
        S: SpatialId,
    {
        self.inner.get_overlapping(target).map(|(flex_id, rank)| {
            let value = self
                .reverse_dictionary
                .get(rank)
                .expect("Dictionary mismatch");
            (flex_id, value)
        })
    }

    /// [`get`](Self::get) と異なり切り取りを行わず、target と重なった[`FlexId`]と値をそのまま返す。
    pub fn remove_overlapping<S: SpatialId>(
        &mut self,
        target: &S,
    ) -> impl Iterator<Item = (FlexId, V)> {
        let removed: Vec<(FlexId, usize)> = self.inner.remove_overlapping(target).collect();
        let results: Vec<(FlexId, V)> = removed
            .into_iter()
            .map(|(flex_id, rank)| {
                let value = self
                    .reverse_dictionary
                    .get(&rank)
                    .expect("Dictionary mismatch")
                    .clone();
                (flex_id, value)
            })
            .collect();
        if !results.is_empty() {
            self.value_index_built = false;
        }
        results.into_iter()
    }

    /// 指定した単体の空間 IDと面で接している[`FlexId`] と値への参照を重複なく返す。入力された空間ID自身と重なる要素は除外する。
    pub fn neighbors_share_face<'a, S: SpatialId>(
        &'a self,
        target: &'a S,
    ) -> impl Iterator<Item = (FlexId, &'a V)> + 'a {
        self.inner
            .neighbors_share_face(target)
            .map(|(flex_id, rank)| {
                let value = self
                    .reverse_dictionary
                    .get(rank)
                    .expect("Dictionary mismatch");
                (flex_id, value)
            })
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
    /// 各 [`SingleId`] には存在時間（時間セル）が付く。
    pub fn flat_single_ids(&self) -> impl Iterator<Item = (SingleId, &V)> + '_ {
        let max_zoomlevel = self.max_zoomlevel().unwrap_or(0);
        self.iter().flat_map(move |(flex_id, value)| {
            let range = RangeId::from(&flex_id);
            let normalized = if range.z() == max_zoomlevel {
                range
            } else {
                range
                    .spatial_children_at_zoom(max_zoomlevel)
                    .expect("target max zoomlevel must be valid")
            };
            normalized
                .single_ids()
                .map(move |single_id| (single_id, value))
        })
    }

    /// コレクション内のすべての値をインプレースで更新します。
    pub fn map_values_in_place<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut V),
    {
        let mut new_dict = BTreeMap::new();
        for (&rank, val) in self.reverse_dictionary.iter_mut() {
            f(val);
            new_dict.insert(val.clone(), rank);
        }
        self.dictionary = new_dict;
        self.value_index_built = false;
    }

    /// `value_index` を `inner` から構築し、上書き等で消えたランクを辞書から取り除く。
    pub fn rebuild_index(&mut self) {
        self.value_index.clear();
        for (flex_id, rank) in self.inner.iter() {
            self.value_index.entry(*rank).or_default().insert(flex_id);
        }
        let live: BTreeSet<usize> = self.value_index.keys().copied().collect();
        self.reverse_dictionary
            .retain(|rank, _| live.contains(rank));
        self.dictionary.retain(|_, rank| live.contains(rank));
        self.value_index_built = true;
    }

    /// 特定の値に対応するすべての時空間[FlexId]を返す。
    pub fn value_get(&self, value: &V) -> impl Iterator<Item = FlexId> + '_ {
        let mut out = Vec::new();
        if let Some(&rank) = self.dictionary.get(value) {
            if self.value_index_built {
                if let Some(set) = self.value_index.get(&rank) {
                    out.extend(set.iter());
                }
            } else {
                for (flex_id, r) in self.inner.iter() {
                    if *r == rank {
                        out.push(flex_id);
                    }
                }
            }
        }
        out.into_iter()
    }

    /// 範囲条件に一致する全ての値の[FlexId]と値への参照を返す。
    pub fn value_range<R: RangeBounds<V>>(
        &self,
        range: R,
    ) -> impl Iterator<Item = (FlexId, &V)> + '_ {
        let wanted: Vec<(&V, usize)> = self.dictionary.range(range).map(|(v, r)| (v, *r)).collect();
        let mut out: Vec<(FlexId, &V)> = Vec::new();
        if self.value_index_built {
            for (val, rank) in &wanted {
                if let Some(set) = self.value_index.get(rank) {
                    out.extend(set.iter().map(|flex_id| (flex_id, *val)));
                }
            }
        } else {
            let lookup: BTreeMap<usize, &V> = wanted.iter().map(|(v, r)| (*r, *v)).collect();
            for (flex_id, rank) in self.inner.iter() {
                if let Some(val) = lookup.get(rank) {
                    out.push((flex_id, *val));
                }
            }
        }
        out.into_iter()
    }

    /// テーブルが空かどうかを返します
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// テーブルに保持されている全ての時空間と値への参照のペアを返します。
    ///
    /// 各空間セルの時間別の値は約数鎖の最小セル列へ分解され、
    /// `(空間セル × 時間セル, 値)` として列挙される。全時間（WHOLE）のセルは
    /// 従来どおり1つの `(FlexId, &V)` になる。
    pub fn iter(&self) -> impl Iterator<Item = (FlexId, &V)> + '_ {
        self.inner.iter().map(move |(flex_id, rank)| {
            let value = self
                .reverse_dictionary
                .get(rank)
                .expect("Dictionary mismatch");
            (flex_id, value)
        })
    }

    /// テーブルに保持されている値への参照を返す。
    pub fn values(&self) -> impl Iterator<Item = &V> + '_ {
        let mut out: Vec<&V> = Vec::new();
        if self.value_index_built {
            out.extend(self.dictionary.keys());
        } else {
            let mut live: BTreeSet<usize> = BTreeSet::new();
            for (_, rank) in self.inner.iter() {
                live.insert(*rank);
            }
            out = live
                .iter()
                .filter_map(|rank| self.reverse_dictionary.get(rank))
                .collect();
            out.sort();
            out.dedup();
        }
        out.into_iter()
    }
}

#[cfg(feature = "persist")]
macro_rules! impl_spatial_id_table_persist {
    ($t:ty) => {
        impl SpatialIdTable<$t> {
            /// この [`SpatialIdTable`] を rkyv バイト列へ直列化する。
            pub fn to_bytes(&self) -> Result<alloc::vec::Vec<u8>, rkyv::rancor::Error> {
                Ok(rkyv::to_bytes::<rkyv::rancor::Error>(self)?.to_vec())
            }

            /// [`to_bytes`](Self::to_bytes) で直列化したバイト列から復元する。
            ///
            /// # Safety
            /// `bytes` は [`SpatialIdTable::to_bytes`] が生成した正当なバイト列でなければならない。
            pub unsafe fn from_bytes(bytes: &[u8]) -> Result<Self, rkyv::rancor::Error> {
                let archived =
                    unsafe { rkyv::access_unchecked::<ArchivedSpatialIdTable<$t>>(bytes) };
                rkyv::deserialize::<Self, rkyv::rancor::Error>(archived)
            }
        }
    };
}

// DB用途・一般的な主要型に対する永続化機能の一括生成
#[cfg(feature = "persist")]
const _: () = {
    // バイト列・文字列
    impl_spatial_id_table_persist!(alloc::vec::Vec<u8>);
    impl_spatial_id_table_persist!(alloc::string::String);

    // 符号付き整数
    impl_spatial_id_table_persist!(i8);
    impl_spatial_id_table_persist!(i16);
    impl_spatial_id_table_persist!(i32);
    impl_spatial_id_table_persist!(i64);
    impl_spatial_id_table_persist!(isize);

    // 符号なし整数
    impl_spatial_id_table_persist!(u8);
    impl_spatial_id_table_persist!(u16);
    impl_spatial_id_table_persist!(u32);
    impl_spatial_id_table_persist!(u64);
    impl_spatial_id_table_persist!(usize);

    // 論理値
    impl_spatial_id_table_persist!(bool);
};
