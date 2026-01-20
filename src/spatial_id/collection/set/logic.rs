use crate::spatial_id::collection::Collection;
use crate::spatial_id::collection::set::memory::SetOnMemory;
use crate::spatial_id::flex_id::FlexId;
use crate::spatial_id::{ToFlexId, collection::set::SetStorage};
use crate::storage::BTreeMapTrait;

#[derive(Default)]
pub struct SetLogic<S: SetStorage + Collection>(S);

impl<S> SetLogic<S>
where
    S: SetStorage + Collection + Default,
{
    pub fn open(set_storage: S) -> Self {
        Self(set_storage)
    }

    pub fn close(self) -> S {
        self.0
    }

    pub fn size(&self) -> usize {
        self.0.main().len()
    }

    pub(crate) fn flex_ids(&self) -> impl Iterator<Item = &FlexId> {
        self.0.flex_ids()
    }

    pub fn insert<I: ToFlexId>(&mut self, target: &I) {
        for flex_id in target.to_flex_id() {
            let mut tmp_set = SetOnMemory::default();
            unsafe { tmp_set.insert_unchecked(&flex_id) };
            for need_insert in tmp_set.flex_ids() {
                for related_rank in self.0.related(&need_insert) {
                    let related_id = self.0.get_flex_id(related_rank).unwrap();

                    //挿入対象のIDが他のIDに完全に含まれる場合
                    if related_id.contains(&flex_id) {
                        return;
                    }
                    //他のIDが挿入対象のIDに完全に含まれる場合
                    else if flex_id.contains(related_id) {
                        self.0.remove_flex_id(related_rank);
                    }
                    //競合解消が必要な場合
                    else {
                    }
                }
            }
        }
    }

    ///重複確認なく挿入を行う
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

    pub fn union(&self, other: &Self) -> SetOnMemory {
        todo!()
    }

    pub fn intersection(&self, other: &Self) -> SetOnMemory {
        todo!()
    }

    pub fn difference(&self, other: &Self) -> SetOnMemory {
        todo!()
    }
}
