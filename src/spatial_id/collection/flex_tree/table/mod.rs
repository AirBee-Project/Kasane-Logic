use std::collections::BTreeMap;
use std::ops::RangeBounds;
pub mod convert;
pub mod json;
pub mod test;

use crate::{FlexId, FlexTreeCore, IntoSingleIds, IterFlexIds, RangeId, SingleId, SpatialIdSet};

/// 値(V)と空間(FlexId)を相互に高速検索・管理するためのテーブル構造。
#[derive(Default, Clone, Debug)]
pub struct SpatialIdTable<V>
where
    V: PartialEq + Ord + Clone,
{
    // メインの空間ツリー (空間 -> Rank)
    inner: FlexTreeCore<usize>,

    // 辞書 (値 -> Rank)
    dictionary: BTreeMap<V, usize>,

    // 逆引き辞書 (Rank -> 値)
    reverse_dictionary: BTreeMap<usize, V>,

    // 逆引きインデックス (Rank -> その値が存在する空間の集合)
    value_index: BTreeMap<usize, SpatialIdSet>,

    // 次に発行する一意なID（Rank）
    current_rank: usize,
}

impl<V> SpatialIdTable<V>
where
    V: PartialEq + Ord + Clone,
{
    /// 空の[SpatialIdTable]を作成します。
    pub fn new() -> Self {
        Self {
            inner: FlexTreeCore::default(),
            dictionary: BTreeMap::default(),
            reverse_dictionary: BTreeMap::default(),
            value_index: BTreeMap::default(),
            current_rank: 0,
        }
    }

    /// 空間に値を挿入します。
    pub fn insert<S: IterFlexIds + Clone>(&mut self, target: S, value: V) {
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

        let mut old_overlaps = Vec::new();
        for (intersected_id, old_rank) in self.inner.get(&target) {
            if old_rank != rank {
                old_overlaps.push((intersected_id, old_rank));
            }
        }

        for (intersected_id, old_rank) in old_overlaps {
            if let Some(old_set) = self.value_index.get_mut(&old_rank) {
                let _ = old_set.remove(&intersected_id);
                if old_set.is_empty() {
                    self.value_index.remove(&old_rank);
                    if let Some(old_val) = self.reverse_dictionary.remove(&old_rank) {
                        self.dictionary.remove(&old_val);
                    }
                }
            }
        }

        self.inner.insert(target.clone(), rank);

        let set = self
            .value_index
            .entry(rank)
            .or_insert_with(SpatialIdSet::new);
        set.insert(target);
    }

    /// 特定の空間（target）と交差するすべての領域と、その値への参照を返します。
    pub fn get<'a, S>(&'a self, target: &'a S) -> impl Iterator<Item = (FlexId, &'a V)> + 'a
    where
        S: IterFlexIds,
    {
        self.inner.get(target).map(|(flex_id, rank)| {
            let value = self.reverse_dictionary.get(&rank).unwrap();
            (flex_id, value)
        })
    }

    /// 指定した空間（target）をツリーからくり抜き、削除された領域とその値を返します。
    pub fn remove<'a, S: IterFlexIds + Clone>(
        &'a mut self,
        target: &'a S,
    ) -> impl Iterator<Item = (FlexId, V)> + 'a {
        let removed_items: Vec<(FlexId, usize)> = self.inner.remove(target).collect();
        let mut results = Vec::new();

        for (flex_id, rank) in removed_items {
            let value = self.reverse_dictionary.get(&rank).unwrap().clone();

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
            let value = self.reverse_dictionary.get(&rank).unwrap();
            RangeId::from(&flex_id)
                .into_single_ids()
                .map(move |single_id| (single_id, value))
        })
    }

    /// 特定の値に対応するすべての[FlexId]を返します。
    pub fn value_get(&self, value: &V) -> impl Iterator<Item = FlexId> + '_ {
        self.dictionary
            .get(value)
            .and_then(|rank| self.value_index.get(rank))
            .into_iter()
            .flat_map(|set| set.iter())
    }

    /// 範囲条件に一致する全ての値の[FlexId]と値への参照のペアを返します。
    pub fn value_range<R: RangeBounds<V>>(
        &self,
        range: R,
    ) -> impl Iterator<Item = (FlexId, &V)> + '_ {
        self.dictionary.range(range).flat_map(move |(val, rank)| {
            let set = self.value_index.get(rank).expect("Index mismatch");
            set.iter().map(move |flex_id| (flex_id, val))
        })
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
                .get(&rank)
                .expect("Dictionary mismatch");
            (flex_id, value)
        })
    }

    /// テーブルに保持されている値への参照を返します。
    pub fn values(&self) -> impl Iterator<Item = &V> + '_ {
        self.dictionary.keys()
    }
}
