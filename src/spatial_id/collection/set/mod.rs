use std::collections::{BTreeMap, btree_map::Entry};

pub mod scanner;

use roaring::{RoaringBitmap, RoaringTreemap};

use crate::{
    FlexId, FlexIdRank, Segment,
    spatial_id::{FlexIds, collection::set::scanner::FlexIdScanPlan},
};

pub struct SetOnMemory {
    f: BTreeMap<Segment, RoaringTreemap>,
    x: BTreeMap<Segment, RoaringTreemap>,
    y: BTreeMap<Segment, RoaringTreemap>,
    main: BTreeMap<FlexIdRank, FlexId>,
    next_rank: u64,
    recycle_rank: Vec<u64>,
}

impl SetOnMemory {
    pub fn new() -> Self {
        SetOnMemory {
            f: BTreeMap::new(),
            x: BTreeMap::new(),
            y: BTreeMap::new(),
            main: BTreeMap::new(),
            next_rank: 0,
            recycle_rank: Vec::new(),
        }
    }

    pub fn insert<T: FlexIds>(&mut self, target: T) {
        let scanner = self.scanner(target);

        //削除が必要なIDを貯めて最後に削除する
        let mut need_delete_ranks = RoaringTreemap::new();

        for flex_id_scanner in scanner.scan() {
            //もし、親に包まれていた場合はそのほかパターンを考える必要がない
            if flex_id_scanner.unique_parent().is_some() {
                continue;
            }

            //必ず削除しなければならないIDを削除予定にする
            need_delete_ranks |= flex_id_scanner.children();

            //競合解消が必要なIDを列挙する
            let partial_overlaps = flex_id_scanner.partial_overlaps();

            //競合解消が不要な場合は次のFLexIDへ行く
            if partial_overlaps.is_empty() {
                continue;
            }

            //Setを作成して競合を解消する
            let mut shave_set = Self::new();
            shave_set.insert(flex_id_scanner.flex_id().clone());

            //SetからPartial Overlapsを順番に削除する
            //この時は常に自分を削る
            for partial_overlap_rank in partial_overlaps {
                //存在しないRankが取れるはずがないのでunwrapする
                let flex_id = self.as_flex_id(&partial_overlap_rank).unwrap();
            }
        }

        //削除が必要なものを削除する
        for nend_delete_rank in need_delete_ranks {
            self.remove_from_rank(nend_delete_rank);
        }
    }

    ///なんのチェックもせず、挿入する
    ///整合性を破壊する可能性があるため、注意して使用すること
    pub unsafe fn uncheck_insert<T: FlexIds>(&mut self, target: T) {
        //ある次元のBTreeMapに対して挿入を行う操作
        let dimension_insert =
            |btree: &mut BTreeMap<Segment, RoaringTreemap>, segment: Segment, rank: FlexIdRank| {
                match btree.entry(segment) {
                    Entry::Vacant(v) => {
                        let mut set = RoaringTreemap::new();
                        set.insert(rank);
                        v.insert(set);
                    }
                    Entry::Occupied(mut o) => {
                        o.get_mut().insert(rank);
                    }
                }
            };

        for flex_id in target.flex_ids() {
            let rank = self.fetch_rank();

            //分離している次元を更新
            dimension_insert(&mut self.f, flex_id.as_f().clone(), rank);
            dimension_insert(&mut self.x, flex_id.as_x().clone(), rank);
            dimension_insert(&mut self.y, flex_id.as_y().clone(), rank);

            //mainに挿入
            self.main.insert(rank, flex_id);
        }
    }

    ///指定した領域を削除して、削除された部分をSetとして返す
    pub fn remove<T: FlexIds>(&mut self, target: T) -> Self {
        let scanner = self.scanner(target);
        let mut result = Self::new();
        for flex_id_scanner in scanner.scan() {
            //もし、親に包まれていた場合はそのほかパターンを考える必要がない
            if flex_id_scanner.unique_parent().is_some() {
                continue;
            }
        }
        todo!()
    }

    ///指定した領域を取得してSetを返す
    pub fn get<T: FlexIds>(&self, target: T) -> Self {
        let scanner = self.scanner(target);
        let mut result = Self::new();
        for flex_id_scanner in scanner.scan() {
            //もし、親に包まれていた場合はそのほかパターンを考える必要がない
            if flex_id_scanner.unique_parent().is_some() {
                unsafe { result.uncheck_insert(flex_id_scanner.flex_id()) };
                continue;
            }

            //子を全て追加する
            for child_rank in flex_id_scanner.children() {
                let flex_id = self.as_flex_id(&child_rank).unwrap();
                unsafe { result.uncheck_insert(flex_id.clone()) };
            }

            //partial_overlapの重なりがある部分を全て追加する
            for partial_overlap_rank in flex_id_scanner.partial_overlaps() {
                let overlap_flex_id = self.as_flex_id(&partial_overlap_rank).unwrap();
                let intersection = overlap_flex_id
                    .intersection(&flex_id_scanner.flex_id())
                    .unwrap();
                unsafe { result.uncheck_insert(intersection) };
            }
        }
        result
    }

    ///新しいRankを予約するためのメソット
    fn fetch_rank(&mut self) -> FlexIdRank {
        match self.recycle_rank.pop() {
            Some(v) => v,
            None => {
                let result = self.next_rank;
                self.next_rank = self.next_rank + 1;
                result
            }
        }
    }

    ///Rankを指定して削除する
    ///存在しないRankをリクエストされた場合はPanicします
    pub fn remove_from_rank(&mut self, rank: FlexIdRank) -> FlexId {
        //特定の次元から削除する
        let dimension_remove =
            |btree: &mut BTreeMap<Segment, RoaringTreemap>, segment: &Segment, rank: FlexIdRank| {
                if let Some(mut entry) = btree.get_mut(segment) {
                    entry.remove(rank);
                    if entry.is_empty() {
                        btree.remove(segment);
                    }
                }
            };

        //存在しないRankをリクエストされた場合はPanicします。
        let flex_id = self.main.remove(&rank).unwrap();
        dimension_remove(&mut self.f, flex_id.as_f(), rank);
        dimension_remove(&mut self.x, flex_id.as_x(), rank);
        dimension_remove(&mut self.y, flex_id.as_y(), rank);

        flex_id
    }

    ///Setの中からFlexIdを効率的にスキャンするようにする
    pub fn scanner<T: FlexIds>(&'_ self, target: T) -> FlexIdScanPlan<'_> {
        FlexIdScanPlan::new(self, target)
    }

    pub fn as_flex_id(&self, rank: &FlexIdRank) -> Option<&FlexId> {
        self.main.get(&rank)
    }

    pub fn f(&self) -> &BTreeMap<Segment, RoaringTreemap> {
        &self.f
    }

    pub fn x(&self) -> &BTreeMap<Segment, RoaringTreemap> {
        &self.x
    }

    pub fn y(&self) -> &BTreeMap<Segment, RoaringTreemap> {
        &self.y
    }
}
