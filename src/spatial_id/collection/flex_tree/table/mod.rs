use crate::spatial_id::collection::flex_tree::core::FlexTreeCore;
use alloc::vec::Vec;

use alloc::collections::{BTreeMap, BTreeSet};
use core::ops::RangeBounds;
pub mod convert;
#[cfg(feature = "json")]
pub mod json;
pub mod test;

use crate::spatial_id::collection::flex_tree::shard_path::ShardPath;
use crate::spatial_id::collection::flex_tree::summary::ShardSummary;
use crate::{AllowedIntervals, FlexId, FlexIdValue, RangeId, SingleId, SpatialId, SpatialIdSet};

/// 値(V)と空間(FlexId)を相互に高速検索・管理するためのテーブル構造。
#[derive(Clone, Debug)]
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

    /// この集合が値を持つ全Segmentを包む最小の[RangeId]を返します。
    pub fn bounding_box(&self) -> Option<RangeId> {
        self.inner.bounding_box()
    }

    /// ランクを格納した内部ツリー。
    pub(crate) fn rank_core(&self) -> &FlexTreeCore<usize> {
        &self.inner
    }

    /// ランクを添字にして実体値を引ける密な表。`[0]` は常に [`None`]（ランクは 1 始まり）。
    ///
    /// 葉ごとに逆引きするなら、`BTreeMap` を葉の数だけ降りるより一度均したほうが速い。
    pub(crate) fn values_by_rank(&self) -> Vec<Option<&V>> {
        let mut by_rank = alloc::vec![None; self.current_rank + 1];
        for (&rank, value) in &self.reverse_dictionary {
            if let Some(slot) = by_rank.get_mut(rank) {
                *slot = Some(value);
            }
        }
        by_rank
    }

    /// ランクのツリーと、ランク順（1 始まり）に並んだ実体値からテーブルを組む。
    ///
    /// `ranks` の各葉は `values` のインデックス + 1 でなければならない。値インデックスは
    /// 未構築（`insert` 直後と同じ状態）で、必要になったときに
    /// [`rebuild_index`](Self::rebuild_index) が組む。
    pub(crate) fn from_ranked_core(ranks: FlexTreeCore<usize>, values: Vec<V>) -> Self {
        let mut dictionary = BTreeMap::new();
        let mut reverse_dictionary = BTreeMap::new();
        for (i, v) in values.into_iter().enumerate() {
            dictionary.insert(v.clone(), i + 1);
            reverse_dictionary.insert(i + 1, v);
        }
        let current_rank = dictionary.len();

        Self {
            inner: ranks,
            dictionary,
            reverse_dictionary,
            value_index: BTreeMap::default(),
            value_index_built: false,
            current_rank,
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

    pub fn get<'a, S>(&'a self, target: &'a S) -> impl Iterator<Item = (FlexId, &'a V)> + 'a
    where
        S: SpatialId,
    {
        self.inner.get(target.clone()).map(|(flex_id, rank)| {
            let value = self.reverse_dictionary.get(&rank).unwrap();
            (flex_id, value)
        })
    }

    /// 特定の範囲（RangeId）と交差するすべての領域と、その値への参照を返します。
    pub fn get_range<'a>(
        &'a self,
        target: &'a crate::RangeId,
    ) -> impl Iterator<Item = (FlexId, &'a V)> + 'a {
        self.inner.range_overlap_ref(target).map(move |(id, rank)| {
            (
                id,
                self.reverse_dictionary
                    .get(rank)
                    .expect("Dictionary mismatch"),
            )
        })
    }

    /// 指定した空間（target）をツリーからくり抜き、削除された領域とその値を返します。
    pub fn remove<S: SpatialId + Clone>(&mut self, target: &S) -> Vec<(FlexId, V)> {
        let removed_items = self.inner.remove(target.clone());
        let mut results = Vec::new();

        for (flex_id, rank) in removed_items {
            let value = self.reverse_dictionary.get(&rank).unwrap().clone();
            results.push((flex_id, value));
        }

        if !results.is_empty() {
            self.value_index_built = false;
        }
        results
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
    pub fn remove_overlapping<S: SpatialId>(&mut self, target: &S) -> Vec<(FlexId, V)> {
        let removed_items = self.inner.remove_overlapping(target.clone());
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

        results
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

    /// このテーブルを、木を走査せずに評価するための要約を作ります。
    pub fn summary(&self) -> ShardSummary {
        self.inner.summary()
    }

    /// このテーブルのシャード木上の位置。定まらない場合は [`None`]。
    pub fn shard_path(&self) -> Option<&ShardPath> {
        self.inner.shard_path()
    }

    /// ツリーの最大ズームレベルを返します。
    pub fn max_zoomlevel(&self) -> Option<u8> {
        self.inner.max_zoomlevel()
    }

    /// 時間方向に結合した [`RangeId`] として読み出す。**空間解像度は変えない**。
    ///
    /// 単位は「その区間を表せる最も粗い秒数」（`gcd(開始秒, 幅)`）。
    /// 単位を選びたい場合は [`range_ids_in`](Self::range_ids_in) を使う。
    pub fn range_ids(&self) -> impl Iterator<Item = (RangeId, &V)> + '_ {
        self.coalesced_range_ids(None)
    }

    /// 時間の単位を [`AllowedIntervals`] の候補から選んで読み出す。
    ///
    /// 候補のうち**その区間を割り切る最も粗いもの**が選ばれる（＝候補の中でSegment数が最小）。
    /// 暦の単位へ正規化したいだけなら `AllowedIntervals::calendar()`
    /// （`temporal_id` feature 有効時のみ）を直接渡せる。
    pub fn range_ids_in<'a>(
        &'a self,
        units: &'a AllowedIntervals,
    ) -> impl Iterator<Item = (RangeId, &'a V)> + use<'a, V> {
        self.coalesced_range_ids(Some(units))
    }

    /// [`flat_single_ids`](Self::flat_single_ids) の、時間単位を指定できる版。
    pub fn flat_single_ids_in<'a>(
        &'a self,
        units: &'a AllowedIntervals,
    ) -> impl Iterator<Item = (SingleId, &'a V)> + use<'a, V> {
        self.expand_range_ids(Some(units))
    }

    /// 内部の rank を値へ引き直しつつ、時間方向に結合した [`RangeId`] を返す。
    ///
    /// 木が持つのは値そのものではなく rank（`usize`）なので、結合は rank のまま行い
    /// （同じ値なら同じ rank なので結合条件は変わらない）、最後に辞書で引き直す。
    fn coalesced_range_ids<'a>(
        &'a self,
        units: Option<&'a AllowedIntervals>,
    ) -> impl Iterator<Item = (RangeId, &'a V)> + use<'a, V> {
        crate::spatial_id::collection::flex_tree::coalesce::coalesce_temporal(
            self.inner
                .iter_ref()
                .map(|(flex_id, rank)| (flex_id, *rank)),
            units,
        )
        .map(move |(range, rank)| {
            let value = self
                .reverse_dictionary
                .get(&rank)
                .expect("Dictionary mismatch");
            (range, value)
        })
    }

    /// [`coalesced_range_ids`](Self::coalesced_range_ids) を単一Segmentの [`SingleId`] へ展開する。
    fn expand_range_ids<'a>(
        &'a self,
        units: Option<&'a AllowedIntervals>,
    ) -> impl Iterator<Item = (SingleId, &'a V)> + use<'a, V> {
        self.coalesced_range_ids(units)
            .flat_map(|(range, value)| range.single_ids().map(move |id| (id, value)))
    }

    /// 最下層の[SingleId]レベルまで展開したイテレータを参照付きで返します。
    ///
    /// 展開の前に、時間方向に隣接する同値のSegmentを結合する。木は時間を2の冪秒のSegmentとして
    /// 持つため、これを行わないと `1800` 秒のような単位で入れた ID が断片のまま出てくる。
    /// 同値かどうかは Rank（値の同一性そのもの）で判定できるので、値の比較は不要。
    pub fn flat_single_ids(&self) -> impl Iterator<Item = (SingleId, &V)> + '_ {
        let merged = crate::spatial_id::collection::flex_tree::coalesce::coalesce_temporal(
            self.inner
                .iter_ref()
                .map(|(flex_id, rank)| (flex_id, *rank)),
            None,
        );

        merged.flat_map(move |(range, rank)| {
            let value = self
                .reverse_dictionary
                .get(&rank)
                .expect("Dictionary mismatch");
            range.single_ids().map(move |single_id| (single_id, value))
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

pub struct SpatialIdTableIntoIter<V: FlexIdValue> {
    inner: crate::spatial_id::collection::flex_tree::core::LeavesIntoIter<usize>,
    reverse_dictionary: alloc::collections::BTreeMap<usize, V>,
}

impl<V: FlexIdValue> Iterator for SpatialIdTableIntoIter<V> {
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

impl<V: FlexIdValue> IntoIterator for SpatialIdTable<V> {
    type Item = (FlexId, V);
    type IntoIter = SpatialIdTableIntoIter<V>;

    fn into_iter(self) -> Self::IntoIter {
        SpatialIdTableIntoIter {
            inner: self.inner.into_iter(),
            reverse_dictionary: self.reverse_dictionary,
        }
    }
}

impl<V: FlexIdValue> FromIterator<(FlexId, V)> for SpatialIdTable<V> {
    fn from_iter<T: IntoIterator<Item = (FlexId, V)>>(iter: T) -> Self {
        let mut table = SpatialIdTable::new();
        for (id, val) in iter {
            table.insert(id, val);
        }
        table
    }
}

impl<V: FlexIdValue> Extend<(FlexId, V)> for SpatialIdTable<V> {
    fn extend<T: IntoIterator<Item = (FlexId, V)>>(&mut self, iter: T) {
        for (id, val) in iter {
            self.insert(id, val);
        }
    }
}

impl<V: FlexIdValue> Default for SpatialIdTable<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> PartialEq for SpatialIdTable<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue + Ord,
{
    fn eq(&self, other: &Self) -> bool {
        // 論理的に等しいテーブルは使われている値の種類数（distinct value数）も一致するはず
        // なので、木を辿る前にO(1)で弾ける安価なガードとして先に見る。
        if self.count() != other.count() || self.dictionary.len() != other.dictionary.len() {
            return false;
        }
        self.iter().eq(other.iter())
    }
}

impl<V> Eq for SpatialIdTable<V> where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue + Ord
{
}
