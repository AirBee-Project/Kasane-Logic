use alloc::vec::Vec;

use alloc::collections::BTreeMap;
use core::ops::RangeBounds;
pub mod convert;
pub mod json;
pub mod test;

use crate::{
    FlexId, FlexTreeCore, IntoSingleIds, IterFlexIds, RangeId, SingleId, SpatialId, SpatialIdSet,
};

/// 値(V)と空間(FlexId)を相互に高速検索・管理するためのテーブル構造。
#[derive(Default, Clone, Debug)]
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
    value_index: BTreeMap<usize, SpatialIdSet>,

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

        let set = self.value_index.entry(rank).or_default();
        set.insert(target);
    }

    /// 大量の空間と値のペアから、Z-Order (Morton Code) ソートとボトムアップ構築（Bulk Loading）を用いて
    /// 既存のテーブルに一括挿入します。テーブルが空の場合は単一の木を一括構築します。
    ///
    /// 内部で `FlexTreeCore::union_with` を使用するため、既存のデータと高速にマージされます。
    /// 入力は重複やオーバーラップを持たないことを推奨します。完全に一致する要素は自動でDedupされます。
    pub fn batch_insert<S, I>(&mut self, iter: I)
    where
        S: crate::IntoFlexIds,
        I: IntoIterator<Item = (S, V)>,
    {
        use crate::spatial_id::collection::flex_tree::core::morton::MortonCode;

        // 1. Convert to FlexId and Value
        let items: Vec<(FlexId, V)> = iter
            .into_iter()
            .flat_map(|(s, v)| s.into_flex_ids().map(move |f| (f, v.clone())))
            .collect();

        if items.is_empty() {
            return;
        }

        // 2. Assign ranks and MortonCode
        let mut batch_items: Vec<(MortonCode, FlexId, usize)> = Vec::with_capacity(items.len());
        for (flex_id, value) in items {
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
            batch_items.push((MortonCode::from_flex_id(&flex_id), flex_id, rank));
        }

        // 3. Sort by Morton Code (Z-Order)
        batch_items.sort_unstable_by_key(|x| x.0);
        batch_items.dedup_by_key(|x| x.0); // Remove exact duplicates to prevent errors

        // 4. Build temporary FlexTreeCore<usize>
        let mut temp_tree = FlexTreeCore::new();
        let (lower, upper) = FlexTreeCore::from_sorted_batch(&batch_items, &temp_tree.empty_leaf);
        temp_tree.lower_root = lower;
        temp_tree.upper_root = upper;

        // 5. Merge into self.inner
        self.inner = self.inner.union_with(&temp_tree, |_, b| *b);

        // 6. Build and merge value_index trees
        batch_items.sort_unstable_by_key(|x| x.2); // Sort by rank
        let mut start_idx = 0;
        while start_idx < batch_items.len() {
            let rank = batch_items[start_idx].2;
            let mut end_idx = start_idx + 1;
            while end_idx < batch_items.len() && batch_items[end_idx].2 == rank {
                end_idx += 1;
            }

            let slice = &batch_items[start_idx..end_idx];
            // Sort back to Morton Code for this specific rank
            let mut rank_items: Vec<_> = slice.iter().map(|(m, f, _)| (*m, f.clone())).collect();
            rank_items.sort_unstable_by_key(|x| x.0);
            rank_items.dedup_by_key(|x| x.0);

            let temp_set = SpatialIdSet::from_sorted_batch(&rank_items);

            let set = self.value_index.entry(rank).or_default();
            // Since SpatialIdSet just wraps FlexTreeCore<()>, we need to expose inner or use insert
            // For now we can just union_with its inner tree. Wait, `union_with` requires `pub` or internal access.
            // Since this is `table`, we can't access `set.inner` without making it pub(crate).
            // Let's just iterate over the temp_set and insert. It's fast enough because `union_with` is not yet on SpatialIdSet.
            // Actually, wait, let's just make `inner` pub(crate) in `SpatialIdSet` or add `union_with` to `SpatialIdSet`.
            // For now, we will add `union_with` to `SpatialIdSet`.
            set.union_with(temp_set);

            start_idx = end_idx;
        }
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
    /// [`get`](Self::get) と異なり切り取りを行わず、target と重なった
    /// [`FlexId`]と値をそのままの返します。
    pub fn get_overlapping<'a, S>(
        &'a self,
        target: &'a S,
    ) -> impl Iterator<Item = (FlexId, &'a V)> + 'a
    where
        S: IterFlexIds,
    {
        self.inner
            .get_overlapping_ref(target)
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
    pub fn remove_overlapping<'a, S: IterFlexIds>(
        &'a mut self,
        target: &'a S,
    ) -> impl Iterator<Item = (FlexId, V)> + 'a {
        let removed_items: Vec<(FlexId, usize)> = self.inner.remove_overlapping(target).collect();
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
                .get(rank)
                .expect("Dictionary mismatch");
            (flex_id, value)
        })
    }

    /// テーブルに保持されている値への参照を返します。
    pub fn values(&self) -> impl Iterator<Item = &V> + '_ {
        self.dictionary.keys()
    }
}
