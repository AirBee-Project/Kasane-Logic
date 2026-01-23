use std::collections::{BTreeMap, HashMap};
use std::fmt::{self, Display};

use roaring::RoaringTreemap;

use crate::spatial_id::collection::set::memory::{SetOnMemory, SetOnMemoryInner};
use crate::spatial_id::collection::{Collection, FlexIdRank, set};
use crate::spatial_id::flex_id::FlexId;
use crate::spatial_id::segment::Segment;
use crate::spatial_id::{ToFlexId, collection::set::SetStorage};

use crate::storage::KeyValueStore;
use crate::{Error, MAX_ZOOM_LEVEL, RangeId, SingleId};

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
        let main: HashMap<FlexIdRank, FlexId> = set_storage.main().iter().collect();

        let flex_id_next_rank = set_storage.move_flex_rank();

        let copy_dim = |source: &S::Dimension| -> BTreeMap<Segment, RoaringTreemap> {
            source.iter().collect()
        };

        let f = copy_dim(set_storage.f());
        let x = copy_dim(set_storage.x());
        let y = copy_dim(set_storage.y());

        let inner = SetOnMemoryInner {
            f,
            x,
            y,
            main,
            flex_id_next_rank,
            flex_id_recycled_ranks: set_storage.move_flex_rank_free_list(),
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

    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    ///内部にあるIDを全てRangeIdとして返す
    pub fn range_ids(&self) -> impl Iterator<Item = RangeId> + '_ {
        self.0.flex_ids().map(|flex_id| flex_id.range_id())
    }

    pub fn single_ids(&self) -> impl Iterator<Item = SingleId> + '_ {
        self.0
            .flex_ids()
            .flat_map(|flex_id| flex_id.range_id().single_ids().collect::<Vec<_>>())
    }

    pub(crate) fn flex_ids(&self) -> Box<dyn Iterator<Item = FlexId> + '_> {
        self.0.flex_ids()
    }

    pub fn max_z(&self) -> u8 {
        let find_max_z_in_dim = |dim: &S::Dimension| -> u8 {
            dim.iter().map(|(seg, _)| seg.to_xy().0).max().unwrap_or(0)
        };

        let f_max = self
            .0
            .f()
            .iter()
            .map(|(s, _)| s.to_f().0)
            .max()
            .unwrap_or(0);

        let x_max = find_max_z_in_dim(self.0.x());
        let y_max = find_max_z_in_dim(self.0.y());

        f_max.max(x_max).max(y_max)
    }

    ///そのSetの中の最も細かい解像度に変換して出力
    pub fn flatten(&self) -> impl Iterator<Item = SingleId> + '_ {
        self.flatten_deep(0).unwrap()
    }

    ///そのSetの中の最も細かい解像度に変換して出力
    /// 元の情報だけを保ったままズームレベルだけを上げる
    /// additional_depthにより、MAX_ZOOM_LEVELまでならさらに細かくできる
    pub fn flatten_deep(
        &self,
        additional_depth: u8,
    ) -> Result<impl Iterator<Item = SingleId> + '_, Error> {
        let current_max = self.max_z();

        let target_z = current_max
            .checked_add(additional_depth)
            .filter(|&z| z <= MAX_ZOOM_LEVEL as u8)
            .ok_or(Error::ZOutOfRange {
                z: current_max.saturating_add(additional_depth),
            })?;

        Ok(self
            .range_ids()
            .flat_map(move |id| {
                let diff = target_z.checked_sub(id.as_z())?;
                let child_range = id.children(diff).ok()?;
                Some(child_range.single_ids().collect::<Vec<_>>())
            })
            .flatten())
    }

    ///重複の解消と結合の最適化を行う
    /// 領域を挿入する。
    pub fn insert<I: ToFlexId>(&mut self, target: &I) {
        let inserts: Vec<FlexId> = target.flex_ids().into_iter().collect();
        for new_id in inserts {
            // Collectionのrelatedを使う
            let related_ranks: Vec<FlexIdRank> = self.0.related(&new_id).into_iter().collect();

            for rank in related_ranks {
                // Collectionのget_flex_idを使う (Option<FlexId>が返る)
                if let Some(existing_id) = self.0.get_flex_id(rank) {
                    if new_id.intersection(&existing_id).is_some() {
                        let existing_backup = existing_id.clone();
                        // Collectionのremove_flex_idを使う
                        self.0.remove_flex_id(rank);

                        let fragments = existing_backup.difference(&new_id);
                        for frag in fragments {
                            unsafe { self.join_insert_unchecked(&frag) };
                        }
                    }
                }
            }
            unsafe { self.join_insert_unchecked(&new_id) };
        }
    }

    ///重複確認なく挿入を行う
    /// 結合の最適化を行わないとEqなどが正常に動作しなくなる
    /// 結合最適化を行ったものを入れないと、ロジックが壊れる
    /// もしくは、明らかに結合不能なIDなど
    pub unsafe fn insert_unchecked<I: ToFlexId>(&mut self, target: &I) {
        for flex_id in target.flex_ids() {
            // Collectionのinsert_flex_idを使う
            self.0.insert_flex_id(&flex_id);
        }
    }

    ///重複確認なく挿入を行う
    ///結合の最適化を行う
    pub unsafe fn join_insert_unchecked<I: ToFlexId>(&mut self, target: &I) {
        for flex_id in target.flex_ids() {
            if let Some(sibling_rank) = self.0.get_f_sibling_flex_id(&flex_id) {
                // get_flex_idは実体を返すので、参照のライフタイム問題はない
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
        for flex_id in target.flex_ids() {
            for related_rank in self.0.related(&flex_id) {
                let related_id = self.0.get_flex_id(related_rank).unwrap();
                unsafe {
                    result.join_insert_unchecked(&flex_id.intersection(&related_id).unwrap())
                };
            }
        }
        result
    }

    ///FlexIdで指定した領域を削除し、削除した領域をSetOnMemoryとして返す
    pub fn remove<I: ToFlexId>(&mut self, target: &I) -> SetOnMemory {
        let mut result = SetOnMemory::default();
        for flex_id in target.flex_ids() {
            for related_rank in self.0.related(&flex_id) {
                let related_id = self.0.get_flex_id(related_rank).unwrap();
                for removed_flex_id in flex_id.difference(&related_id) {
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
            unsafe { result.join_insert_unchecked(&id) };
        }
        for flex_id in merger.flex_ids() {
            result.insert(&flex_id);
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
            // Collectionのrelated
            let related_ranks = searcher.0.related(&scan_id);

            for rank in related_ranks {
                if let Some(searcher_id) = searcher.0.get_flex_id(rank) {
                    if let Some(inter) = scan_id.intersection(&searcher_id) {
                        unsafe { result.join_insert_unchecked(&inter) };
                    }
                }
            }
        }

        result
    }
    pub fn difference(&self, other: &Self) -> SetOnMemory {
        if other.is_empty() {
            return Self::load(&self.0);
        }
        if self.is_empty() {
            return SetOnMemory::default();
        }

        let mut result = SetOnMemory::default();

        for self_id in self.flex_ids() {
            let mut fragments = vec![self_id.clone()];
            let related_ranks = other.0.related(&self_id);

            for rank in related_ranks {
                if let Some(other_id) = other.0.get_flex_id(rank) {
                    let mut next_fragments = Vec::with_capacity(fragments.len());

                    for frag in fragments {
                        if frag.intersection(&other_id).is_some() {
                            let diffs = frag.difference(&other_id);
                            next_fragments.extend(diffs);
                        } else {
                            next_fragments.push(frag);
                        }
                    }
                    fragments = next_fragments;

                    if fragments.is_empty() {
                        break;
                    }
                } else {
                    panic!()
                }
            }
            for frag in fragments {
                unsafe { result.join_insert_unchecked(&frag) };
            }
        }

        result
    }

    ///二つのSetの表す空間的な範囲が等しいかどうかを見る
    /// コストはそこそこ高い
    pub fn equal(&self, other: &Self) -> bool {
        if self.size() != other.size() {
            return false;
        }
        // 実体を返すようになったので Vec<FlexId> になる
        let mut self_ids: Vec<FlexId> = self.flex_ids().collect();
        let mut other_ids: Vec<FlexId> = other.flex_ids().collect();
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

impl<S> Display for SetLogic<S>
where
    S: SetStorage + Collection + Default,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for range_id in self.range_ids() {
            write!(f, "{},", range_id)?;
        }
        Ok(())
    }
}
