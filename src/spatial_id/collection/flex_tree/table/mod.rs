use alloc::vec::Vec;

use alloc::collections::{BTreeMap, BTreeSet};
use core::ops::RangeBounds;
pub mod convert;
#[cfg(feature = "json")]
pub mod json;
pub mod persist;
pub mod test;

use crate::{CellValue, FlexId, FlexTreeCore, RangeId, SingleId, SpatialId, SpatialIdSet};

/// 値(V)と空間(FlexId)を相互に高速検索・管理するためのテーブル構造。
#[derive(Clone, Debug)]
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
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue + Ord,
{
    // メインの空間ツリー (空間 -> Rank)
    inner: FlexTreeCore<usize>,

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
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue + Ord,
{
    /// 空の[SpatialIdTable]を作成します。
    pub fn new() -> Self {
        Self {
            inner: FlexTreeCore::default(),
            dictionary: BTreeMap::default(),
            reverse_dictionary: BTreeMap::default(),
            value_index: BTreeMap::default(),
            value_index_built: true,
            current_rank: 0,
        }
    }

    /// 空間に値を挿入します。
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

        self.inner.insert(target, rank);
        self.value_index_built = false;
    }

    /// 特定の空間（target）と交差するすべての領域と、その値への参照を返します。
    pub fn get<'a, S>(&'a self, target: &'a S) -> impl Iterator<Item = (FlexId, &'a V)> + 'a
    where
        S: SpatialId,
    {
        self.inner.get(target.clone()).map(|(flex_id, rank)| {
            let value = self.reverse_dictionary.get(&rank).unwrap();
            (flex_id, value)
        })
    }

    /// 指定した空間（target）をツリーからくり抜き、削除された領域とその値を返します。
    pub fn remove<'a, S: SpatialId + Clone>(
        &'a mut self,
        target: &'a S,
    ) -> impl Iterator<Item = (FlexId, V)> + 'a {
        let removed_items: Vec<(FlexId, usize)> = self.inner.remove(target.clone()).collect();
        let mut results = Vec::new();

        for (flex_id, rank) in removed_items {
            let value = self.reverse_dictionary.get(&rank).unwrap().clone();
            results.push((flex_id, value));
        }

        if !results.is_empty() {
            self.value_index_built = false;
        }
        results.into_iter()
    }
    /// [`get`](Self::get) と異なり切り取りを行わず、target と重なった
    /// [`FlexId`]と値をそのままの返します。
    pub fn get_overlapping<'a, S>(
        &'a self,
        target: &'a S,
    ) -> impl Iterator<Item = (FlexId, &'a V)> + 'a
    where
        S: SpatialId,
    {
        self.inner
            .get_overlapping_ref(target.clone())
            .map(|(flex_id, rank)| {
                let value = self
                    .reverse_dictionary
                    .get(rank)
                    .expect("Dictionary mismatch");
                (flex_id, value)
            })
    }

    /// [`get`](Self::get) と異なり切り取りを行わず、target と重なった
    /// [`FlexId`]と値をそのままの返します。
    pub fn remove_overlapping<'a, S: SpatialId>(
        &'a mut self,
        target: &'a S,
    ) -> impl Iterator<Item = (FlexId, V)> + 'a {
        let removed_items: Vec<(FlexId, usize)> =
            self.inner.remove_overlapping(target.clone()).collect();
        let mut results = Vec::new();

        for (flex_id, rank) in removed_items {
            let value = self
                .reverse_dictionary
                .get(&rank)
                .expect("Dictionary mismatch")
                .clone();

            if let Some(set) = self.value_index.get_mut(&rank) {
                let _ = set.remove(&flex_id);

                if set.is_empty() {
                    self.value_index.remove(&rank);
                    self.reverse_dictionary.remove(&rank);
                    self.dictionary.remove(&value);
                }
            }
            results.push((flex_id, value));
        }

        results.into_iter()
    }

    /// 指定した単体の空間 IDと面で接している[`FlexId`] と値への参照を重複なく返します。入力された空間ID自身と重なる要素は除外します。
    pub fn neighbors_share_face<'a, S: SpatialId>(
        &'a self,
        target: &S,
    ) -> impl Iterator<Item = (FlexId, &'a V)> + 'a {
        self.inner
            .neighbors_share_face_ref(target)
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
    pub fn flat_single_ids(&self) -> impl Iterator<Item = (SingleId, &V)> + '_ {
        self.inner.iter_ref().flat_map(|(flex_id, rank)| {
            let value = self.reverse_dictionary.get(rank).unwrap();
            RangeId::from(&flex_id)
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
            self.value_index.entry(rank).or_default().insert(flex_id);
        }
        let live: BTreeSet<usize> = self.value_index.keys().copied().collect();
        self.reverse_dictionary
            .retain(|rank, _| live.contains(rank));
        self.dictionary.retain(|_, rank| live.contains(rank));
        self.value_index_built = true;
    }

    /// 特定の値に対応するすべての[FlexId]を返す。
    pub fn value_get(&self, value: &V) -> impl Iterator<Item = FlexId> + '_ {
        let mut out = Vec::new();
        if let Some(&rank) = self.dictionary.get(value) {
            if self.value_index_built {
                if let Some(set) = self.value_index.get(&rank) {
                    out.extend(set.iter());
                }
            } else {
                for (flex_id, r) in self.inner.iter() {
                    if r == rank {
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
                if let Some(val) = lookup.get(&rank) {
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

    /// テーブルに保持されている全ての空間と値への参照のペアを返します。
    pub fn iter(&self) -> impl Iterator<Item = (FlexId, &V)> + '_ {
        self.inner.iter_ref().map(move |(flex_id, rank)| {
            let value = self
                .reverse_dictionary
                .get(rank)
                .expect("Dictionary mismatch");
            (flex_id, value)
        })
    }

    #[cfg(feature = "rayon")]
    pub fn par_iter(&self) -> impl rayon::iter::ParallelIterator<Item = (FlexId, &V)> + '_ {
        use rayon::prelude::*;
        self.inner.par_iter().map(move |(flex_id, rank)| {
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
                live.insert(rank);
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

pub struct SpatialIdTableIntoIter<V: CellValue> {
    inner: crate::spatial_id::collection::flex_tree::core::LeavesIntoIter<usize>,
    reverse_dictionary: alloc::collections::BTreeMap<usize, V>,
}

impl<V: CellValue> Iterator for SpatialIdTableIntoIter<V> {
    type Item = (FlexId, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(flex_id, rank)| {
            let value = self
                .reverse_dictionary
                .get(&rank)
                .expect("Dictionary mismatch")
                .clone();
            (flex_id, value)
        })
    }
}

impl<V: CellValue> IntoIterator for SpatialIdTable<V> {
    type Item = (FlexId, V);
    type IntoIter = SpatialIdTableIntoIter<V>;

    fn into_iter(self) -> Self::IntoIter {
        SpatialIdTableIntoIter {
            inner: self.inner.into_iter(),
            reverse_dictionary: self.reverse_dictionary,
        }
    }
}

impl<V: CellValue> FromIterator<(FlexId, V)> for SpatialIdTable<V> {
    fn from_iter<T: IntoIterator<Item = (FlexId, V)>>(iter: T) -> Self {
        let mut table = SpatialIdTable::new();
        for (id, val) in iter {
            table.insert(id, val);
        }
        table
    }
}

impl<V: CellValue> Extend<(FlexId, V)> for SpatialIdTable<V> {
    fn extend<T: IntoIterator<Item = (FlexId, V)>>(&mut self, iter: T) {
        for (id, val) in iter {
            self.insert(id, val);
        }
    }
}

impl<V: CellValue> Default for SpatialIdTable<V> {
    fn default() -> Self {
        Self::new()
    }
}
