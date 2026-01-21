use std::collections::BTreeMap;

use roaring::RoaringTreemap;

use crate::RangeId;
use crate::spatial_id::collection::set::memory::{SetOnMemory, SetOnMemoryInner};
use crate::spatial_id::collection::{Collection, FlexIdRank, set};
use crate::spatial_id::flex_id::FlexId;
use crate::spatial_id::segment::Segment;
use crate::spatial_id::{ToFlexId, collection::set::SetStorage};
use crate::storage::BTreeMapTrait;

#[derive(Default)]
pub struct SetLogic<S: SetStorage + Collection>(pub(crate) S);

impl<S> SetLogic<S>
where
    S: SetStorage + Collection + Default,
{
    ///SetStorageが実装された型を開いて、操作可能な状態にする
    pub fn open(set_storage: S) -> Self {
        Self(set_storage)
    }

    ///SetStorageが実装された型を読み込んで、コピーのSetOnMemoryを作成する
    ///大量のReadが発生する可能性があるため、注意して使用せよ
    pub fn load(set_storage: &S) -> SetOnMemory
    where
        S: SetStorage + Collection,
    {
        let main: BTreeMap<FlexIdRank, FlexId> = set_storage
            .main()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let next_rank = main.keys().next_back().map(|&r| r + 1).unwrap_or(0);
        let copy_dim = |source: &S::Dimension| -> BTreeMap<Segment, RoaringTreemap> {
            source.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        };
        let f = copy_dim(set_storage.f());
        let x = copy_dim(set_storage.x());
        let y = copy_dim(set_storage.y());
        let inner = SetOnMemoryInner {
            f,
            x,
            y,
            main,
            next_rank,
            recycled_ranks: Vec::new(),
        };
        SetOnMemory(SetLogic::open(inner))
    }

    ///SetStorageが実装された型を外に出す
    pub fn close(self) -> S {
        self.0
    }

    ///内部にあるFlexIdの個数を返す
    pub fn size(&self) -> usize {
        self.0.main().len()
    }

    ///内部にあるRangeIdを全て返す
    pub fn range_ids(&self) -> impl Iterator<Item = RangeId> + '_ {
        self.0.main().iter().map(|flex_id| flex_id.1.decode())
    }

    ///内部にあるFlexIdを全て返す
    pub fn flex_ids(&self) -> impl Iterator<Item = &FlexId> + '_ {
        self.0.main().iter().map(|f| f.1)
    }

    ///重複の解消と結合の最適化を行う
    pub fn insert<I: ToFlexId>(&mut self, target: &I) {
        let mut work_list: Vec<FlexId> = target.to_flex_id().into_iter().collect();

        'process_queue: while let Some(current_insert) = work_list.pop() {
            let related_ranks = self.0.related(&current_insert);

            for rank in related_ranks {
                if let Some(existing_id) = self.0.get_flex_id(rank) {
                    if existing_id.contains(&current_insert) {
                        continue 'process_queue;
                    } else if current_insert.contains(existing_id) {
                        self.0.remove_flex_id(rank);
                    } else {
                        let fragments = current_insert.difference(existing_id);
                        work_list.extend(fragments);
                        continue 'process_queue;
                    }
                }
            }
            unsafe { self.join_insert_unchecked(&current_insert) };
        }
    }

    ///重複確認なく挿入を行う
    /// 結合の最適化を行わないとEqなどが正常に動作しなくなる
    /// 結合最適化を行ったものを入れないと、ロジックが壊れる
    /// もしくは、明らかに結合不能なIDなど
    pub unsafe fn insert_unchecked<I: ToFlexId>(&mut self, target: &I) {
        for flex_id in target.to_flex_id() {
            self.0.insert_flex_id(&flex_id);
        }
    }

    ///重複確認なく挿入を行う
    ///結合の最適化を行う
    pub unsafe fn join_insert_unchecked<I: ToFlexId>(&mut self, target: &I) {
        for flex_id in target.to_flex_id() {
            if let Some(sibling_rank) = self.0.get_f_sibling_flex_id(&flex_id) {
                if let Some(parent) = self.0.get_flex_id(sibling_rank).unwrap().f_parent() {
                    self.0.remove_flex_id(sibling_rank);
                    unsafe { self.join_insert_unchecked(&parent) };
                    continue;
                }
            }

            if let Some(sibling_rank) = self.0.get_x_sibling_flex_id(&flex_id) {
                if let Some(parent) = self.0.get_flex_id(sibling_rank).unwrap().x_parent() {
                    self.0.remove_flex_id(sibling_rank);
                    unsafe { self.join_insert_unchecked(&parent) };
                    continue;
                }
            }

            if let Some(sibling_rank) = self.0.get_y_sibling_flex_id(&flex_id) {
                if let Some(parent) = self.0.get_flex_id(sibling_rank).unwrap().y_parent() {
                    self.0.remove_flex_id(sibling_rank);
                    unsafe { self.join_insert_unchecked(&parent) };
                    continue;
                }
            }
            self.0.insert_flex_id(&flex_id);
        }
    }

    ///FlexIdで指定した領域を取得し、削除した領域をSetOnMemoryとして返す
    pub fn get<I: ToFlexId>(&mut self, target: &I) -> SetOnMemory {
        let mut result = SetOnMemory::default();
        for flex_id in target.to_flex_id() {
            for related_rank in self.0.related(&flex_id) {
                let related_id = self.0.get_flex_id(related_rank).unwrap();
                unsafe { result.join_insert_unchecked(&flex_id.intersection(related_id).unwrap()) };
            }
        }
        result
    }

    ///FlexIdで指定した領域を削除し、削除した領域をSetOnMemoryとして返す
    pub fn remove<I: ToFlexId>(&mut self, target: &I) -> SetOnMemory {
        let mut result = SetOnMemory::default();
        for flex_id in target.to_flex_id() {
            for related_rank in self.0.related(&flex_id) {
                let related_id = self.0.get_flex_id(related_rank).unwrap();
                for removed_flex_id in flex_id.difference(related_id) {
                    unsafe { result.join_insert_unchecked(&removed_flex_id) };
                }
            }
        }
        result
    }

    ///2つのSetの和集合のSetを作成する
    pub fn union(&self, other: &Self) -> SetOnMemory {
        let mut result = SetOnMemory::default();
        let (base, merger) = if self.size() >= other.size() {
            (self, other)
        } else {
            (other, self)
        };
        for id in base.flex_ids() {
            unsafe { result.join_insert_unchecked(id) };
        }
        for flex_id in merger.flex_ids() {
            result.insert(flex_id);
        }
        result
    }

    ///2つのSetの積集合のSetを作成する
    pub fn intersection(&self, other: &Self) -> SetOnMemory {
        let mut result = SetOnMemory::default();
        let (scanner, searcher) = if self.size() < other.size() {
            (self, other)
        } else {
            (other, self)
        };
        for scan_id in scanner.flex_ids() {
            let related_ranks = searcher.0.related(scan_id);

            for rank in related_ranks {
                if let Some(searcher_id) = searcher.0.get_flex_id(rank) {
                    if let Some(inter) = scan_id.intersection(searcher_id) {
                        unsafe { result.join_insert_unchecked(&inter) };
                    }
                }
            }
        }

        result
    }
    ///2つのSetの差集合のSetを作成する
    pub fn difference(&self, other: &Self) -> SetOnMemory {
        let mut result = SetOnMemory::default();

        //引く側が十分に小さい場合
        if other.size() < self.size() * 2 {
            for id in self.flex_ids() {
                unsafe { result.join_insert_unchecked(id) };
            }
            // B を削除
            for need_remove in other.flex_ids() {
                result.remove(need_remove);
            }
        } else {
            for self_id in self.flex_ids() {
                let mut fragments = vec![self_id.clone()];
                let related_ranks = other.0.related(self_id);
                for rank in related_ranks {
                    let other_id = other.0.get_flex_id(rank).unwrap();
                    let mut next_fragments = Vec::new();
                    for frag in fragments {
                        let diffs = frag.difference(other_id);
                        next_fragments.extend(diffs);
                    }
                    fragments = next_fragments;
                    if fragments.is_empty() {
                        break;
                    }
                }
                for frag in fragments {
                    unsafe { result.join_insert_unchecked(&frag) };
                }
            }
        }
        result
    }

    ///二つのSetの表す空間的な範囲が等しいかどうかを見る
    /// コストはそこそこ高い
    fn equal(&self, other: &Self) -> bool {
        if self.size() != other.size() {
            return false;
        }
        let mut self_ids: Vec<&FlexId> = self.flex_ids().collect();
        let mut other_ids: Vec<&FlexId> = other.flex_ids().collect();
        self_ids.sort();
        other_ids.sort();
        self_ids == other_ids
    }

    ///全てのFlexIdをSingleIdに変換して、2つのSetの中身が完全に一致することを検証します。
    ///主にテスト用です。重いのでプロダクションでは使用しないでください。
    #[cfg(debug_assertions)]
    pub fn verification_eq(&self, other: &Self) -> bool {
        use crate::SingleId;
        let expand_to_singles = |set: &Self| -> Vec<SingleId> {
            let mut singles: Vec<SingleId> = set
                .range_ids()
                .collect::<Vec<_>>()
                .into_iter()
                .flat_map(|range_id| range_id.single_ids().collect::<Vec<_>>())
                .collect();
            singles.sort();
            singles
        };

        let self_singles = expand_to_singles(self);
        let other_singles = expand_to_singles(other);

        self_singles == other_singles
    }
}
