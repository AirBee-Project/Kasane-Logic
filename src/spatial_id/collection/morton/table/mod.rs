use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;
use core::ops::RangeBounds;

use crate::{CellValue, IterFlexIds, SingleId, SpatialId};

use super::core::MortonCore;

pub mod convert;
pub mod json;

/// 値(V)と空間を相互に管理する Morton order バックエンドのテーブル構造。
///
/// 空間→ランク(usize) を [`MortonCore`] に持ち、ランク↔値の辞書で値を引く構造は
/// FlexTree 版と同じ。公開 API は単一解像度の [`SingleId`] を返す。
#[derive(Default, Clone, Debug)]
pub struct SpatialIdTable<V>
where
    V: CellValue,
{
    /// メインの空間マップ（空間 -> ランク）。
    inner: MortonCore<usize>,

    /// 辞書（値 -> ランク）。
    dictionary: BTreeMap<V, usize>,

    /// 逆引き辞書（ランク -> 値）。
    reverse_dictionary: BTreeMap<usize, V>,

    /// 逆引きインデックス（ランク -> その値が存在する空間の集合）。
    value_index: BTreeMap<usize, super::set::SpatialIdSet>,

    /// `value_index` が `inner` と整合しているか。
    value_index_built: bool,

    /// 次に発行する一意なランク。
    current_rank: usize,
}

impl<V> SpatialIdTable<V>
where
    V: CellValue,
{
    /// 空の [`SpatialIdTable`] を作成します。
    pub fn new() -> Self {
        Self {
            inner: MortonCore::new(),
            dictionary: BTreeMap::new(),
            reverse_dictionary: BTreeMap::new(),
            value_index: BTreeMap::new(),
            value_index_built: true,
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

        self.inner.insert(target, rank);
        self.value_index_built = false;
    }

    /// 特定の空間（target）と交差するすべての領域と、その値への参照を返します。
    pub fn get<'a, S>(&'a self, target: &'a S) -> impl Iterator<Item = (SingleId, &'a V)> + 'a
    where
        S: IterFlexIds,
    {
        self.inner
            .get_single(target)
            .into_iter()
            .map(|(sid, rank)| {
                let value = self.reverse_dictionary.get(&rank).expect("rank mismatch");
                (sid, value)
            })
    }

    /// 指定した空間（target）をくり抜き、削除された領域とその値を返します。
    pub fn remove<S: IterFlexIds + Clone>(&mut self, target: &S) -> Vec<(SingleId, V)> {
        let removed: Vec<(SingleId, usize)> = self.inner.remove(target);
        let mut results = Vec::new();
        for (sid, rank) in removed {
            let value = self.reverse_dictionary.get(&rank).expect("rank").clone();
            results.push((sid, value));
        }
        if !results.is_empty() {
            self.value_index_built = false;
        }
        results
    }

    /// 切り取りを行わず、target と重なった [`SingleId`] と値への参照をそのまま返します。
    pub fn get_overlapping<'a, S>(
        &'a self,
        target: &'a S,
    ) -> impl Iterator<Item = (SingleId, &'a V)> + 'a
    where
        S: IterFlexIds,
    {
        self.inner
            .get_overlapping_single(target)
            .into_iter()
            .map(|(sid, rank)| {
                let value = self.reverse_dictionary.get(&rank).expect("rank mismatch");
                (sid, value)
            })
    }

    /// 切り取りを行わず、target と重なった [`SingleId`] と値を丸ごと取り除いて返します。
    pub fn remove_overlapping<S: IterFlexIds>(&mut self, target: &S) -> Vec<(SingleId, V)> {
        let removed = self.inner.remove_overlapping(target);
        if !removed.is_empty() {
            self.value_index_built = false;
        }
        removed
            .into_iter()
            .map(|(sid, rank)| {
                let value = self.reverse_dictionary.get(&rank).expect("rank").clone();
                (sid, value)
            })
            .collect()
    }

    /// 入力した単体の空間IDと面で接している [`SingleId`] と値への参照を重複なく返します。
    pub fn neighbors_share_face<'a, S: SpatialId>(
        &'a self,
        target: &S,
    ) -> impl Iterator<Item = (SingleId, &'a V)> + 'a {
        self.inner
            .neighbors_share_face_single(target)
            .into_iter()
            .map(|(sid, rank)| {
                let value = self.reverse_dictionary.get(&rank).expect("rank mismatch");
                (sid, value)
            })
    }

    /// 保持している [`SingleId`] の総数を返します。
    pub fn count(&self) -> usize {
        self.inner.count()
    }

    /// 最大ズームレベルを返します。
    pub fn max_zoomlevel(&self) -> Option<u8> {
        self.inner.max_zoomlevel()
    }

    /// 全体の最大ズームへ揃えた [`SingleId`] と値への参照を返します。
    pub fn flat_single_ids(&self) -> impl Iterator<Item = (SingleId, &V)> + '_ {
        self.inner.flat_single_ids().into_iter().map(|(sid, rank)| {
            let value = self.reverse_dictionary.get(&rank).expect("rank mismatch");
            // 借用を満たすため rank を引いた参照を返す。
            (sid, value)
        })
    }

    /// `value_index` を `inner` から構築し、消えたランクを辞書から取り除く。
    pub fn rebuild_index(&mut self) {
        self.value_index.clear();
        for (sid, rank) in self.inner.iter_single().map(|(s, r)| (s, *r)) {
            self.value_index.entry(rank).or_default().insert(sid);
        }
        let live: BTreeSet<usize> = self.value_index.keys().copied().collect();
        self.reverse_dictionary
            .retain(|rank, _| live.contains(rank));
        self.dictionary.retain(|_, rank| live.contains(rank));
        self.value_index_built = true;
    }

    /// 特定の値に対応するすべての [`SingleId`] を返します。
    pub fn value_get(&self, value: &V) -> impl Iterator<Item = SingleId> + '_ {
        let mut out = Vec::new();
        if let Some(&rank) = self.dictionary.get(value) {
            if self.value_index_built {
                if let Some(set) = self.value_index.get(&rank) {
                    out.extend(set.iter());
                }
            } else {
                for (sid, r) in self.inner.iter_single() {
                    if *r == rank {
                        out.push(sid);
                    }
                }
            }
        }
        out.into_iter()
    }

    /// 範囲条件に一致する全ての値の [`SingleId`] と値への参照を返します。
    pub fn value_range<R: RangeBounds<V>>(
        &self,
        range: R,
    ) -> impl Iterator<Item = (SingleId, &V)> + '_ {
        let wanted: Vec<(&V, usize)> = self.dictionary.range(range).map(|(v, r)| (v, *r)).collect();
        let mut out: Vec<(SingleId, &V)> = Vec::new();
        if self.value_index_built {
            for (val, rank) in &wanted {
                if let Some(set) = self.value_index.get(rank) {
                    out.extend(set.iter().map(|sid| (sid, *val)));
                }
            }
        } else {
            let lookup: BTreeMap<usize, &V> = wanted.iter().map(|(v, r)| (*r, *v)).collect();
            for (sid, rank) in self.inner.iter_single() {
                if let Some(val) = lookup.get(rank) {
                    out.push((sid, *val));
                }
            }
        }
        out.into_iter()
    }

    /// テーブルが空かどうかを返します。
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// 全ての空間と値への参照のペアを返します。
    pub fn iter(&self) -> impl Iterator<Item = (SingleId, &V)> + '_ {
        self.inner.iter_single().map(move |(sid, rank)| {
            let value = self.reverse_dictionary.get(rank).expect("rank mismatch");
            (sid, value)
        })
    }

    /// 保持している値への参照を返します。
    pub fn values(&self) -> impl Iterator<Item = &V> + '_ {
        let mut out: Vec<&V> = Vec::new();
        if self.value_index_built {
            out.extend(self.dictionary.keys());
        } else {
            let mut live: BTreeSet<usize> = BTreeSet::new();
            for (_, rank) in self.inner.iter_single() {
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
