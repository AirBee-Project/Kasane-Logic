use std::collections::{BTreeMap, HashMap, btree_map::Entry};

pub mod tests;

use roaring::RoaringTreemap;

use crate::{
    FlexId, FlexIdRank, RangeId, Segment, SingleId,
    spatial_id::{FlexIds, collection::scanner::Scanner, helpers::fast_intersect},
};

#[derive(Clone, Debug, Default)]
pub struct SetOnMemory {
    f: BTreeMap<Segment, RoaringTreemap>,
    x: BTreeMap<Segment, RoaringTreemap>,
    y: BTreeMap<Segment, RoaringTreemap>,
    main: HashMap<FlexIdRank, FlexId>,
    next_rank: u64,
    recycle_rank: Vec<u64>,
}

impl<'a> Scanner<'a> for &'a SetOnMemory {
    fn f(&self) -> &'a BTreeMap<Segment, RoaringTreemap> {
        &self.f
    }

    fn x(&self) -> &'a BTreeMap<Segment, RoaringTreemap> {
        &self.x
    }

    fn y(&self) -> &'a BTreeMap<Segment, RoaringTreemap> {
        &self.y
    }

    fn main(&self) -> &'a HashMap<FlexIdRank, FlexId> {
        &self.main
    }
}

impl SetOnMemory {
    const RECYCLE_RANK_MAX: usize = 1024;

    pub fn new() -> Self {
        SetOnMemory {
            f: BTreeMap::new(),
            x: BTreeMap::new(),
            y: BTreeMap::new(),
            main: HashMap::new(),
            next_rank: 0,
            recycle_rank: Vec::new(),
        }
    }

    pub fn insert<T: FlexIds>(&mut self, target: &T) {
        let ref_self = &*self;
        let scanner = ref_self.flex_id_scan_plan(target.clone());
        //削除が必要なIDを貯めて最後に削除する
        let mut need_delete_ranks = RoaringTreemap::new();
        let mut need_insert_flex_ids: Vec<FlexId> = Vec::new();

        for flex_id_scanner in scanner.scan() {
            //もし、親に包まれていた場合はそのほかパターンを考える必要がない
            if flex_id_scanner.parent().is_some() {
                continue;
            }

            //必ず削除しなければならないIDを削除予定にする
            need_delete_ranks |= flex_id_scanner.children();

            //競合解消が必要なIDを列挙する
            let partial_overlaps = flex_id_scanner.partial_overlaps();

            //競合解消が不要な場合は次のFLexIDへ行く
            if partial_overlaps.is_empty() {
                need_insert_flex_ids.push(flex_id_scanner.flex_id().clone());
                continue;
            }

            //Setを作成して競合を解消する
            let mut shave_set = Self::new();
            unsafe { shave_set.uncheck_insert(flex_id_scanner.flex_id().clone()) };

            //SetからPartial Overlapsを順番に削除する
            //この時は常に自分を削る
            for partial_overlap_rank in partial_overlaps {
                //存在しないRankが取れるはずがないのでunwrapする
                let flex_id = self.as_flex_id(&partial_overlap_rank).unwrap();
                shave_set.remove(flex_id);
            }

            //削って競合がなくなったSetを挿入する
            need_insert_flex_ids.extend(shave_set.as_flex_ids().cloned());
        }

        //削除が必要なものを削除する
        for nend_delete_rank in need_delete_ranks {
            self.remove_from_rank(nend_delete_rank);
        }

        //挿入するべきものを挿入する
        for need_insert_flex_id in need_insert_flex_ids {
            unsafe { self.join_uncheck_insert(need_insert_flex_id) };
        }
    }

    ///なんのチェックもせず、挿入する
    ///整合性を破壊する可能性があるため、注意して使用すること
    /// 隣にあるIDと結合されることがある
    pub unsafe fn join_uncheck_insert<T: FlexIds>(&mut self, target: T) {
        for flex_id in target.flex_ids() {
            match self.find(flex_id.f_sibling()) {
                Some(v) => match flex_id.f_parent() {
                    Some(parent) => {
                        self.remove_from_rank(v);
                        unsafe { self.join_uncheck_insert(parent) };
                        continue;
                    }
                    None => {}
                },
                None => {}
            }

            // X軸方向の結合チェック
            match self.find(flex_id.x_sibling()) {
                Some(v) => {
                    self.remove_from_rank(v);
                    unsafe { self.join_uncheck_insert(flex_id.x_parent().unwrap()) };
                    continue;
                }
                None => {}
            }

            // Y軸方向の結合チェック
            match self.find(flex_id.y_sibling()) {
                Some(v) => {
                    self.remove_from_rank(v);
                    unsafe { self.join_uncheck_insert(flex_id.y_parent().unwrap()) };
                    continue;
                }
                None => {}
            }

            // 結合相手がいなければ、そのまま挿入
            unsafe { self.uncheck_insert(flex_id) };
        }
    }

    ///Targetと全く同じ形のFlexIdを見つけ、そのFlexIdRankを返す
    pub fn find(&self, target: FlexId) -> Option<FlexIdRank> {
        let f = self.f.get(target.as_f())?;
        let x = self.x.get(target.as_x())?;
        let y = self.y.get(target.as_y())?;
        fast_intersect([f, x, y]).iter().next().clone()
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
    /// 指定されたID集合を削除する
    pub fn remove<T: FlexIds>(&mut self, target: &T) {
        for flex_id in target.flex_ids() {
            let ref_self = &*self;
            let scanner = ref_self.flex_id_scan_plan(target.clone());

            let mut need_delete_ranks: Vec<FlexIdRank> = Vec::new();
            let mut need_insert_flex_ids: Vec<FlexId> = Vec::new();

            for scan_result in scanner.scan() {
                if let Some(parent_rank) = scan_result.parent() {
                    if let Some(parent_flex_id) = self.as_flex_id(&parent_rank) {
                        let diff = parent_flex_id.difference(&flex_id);
                        need_delete_ranks.push(parent_rank);
                        need_insert_flex_ids.extend(diff);
                    }
                } else {
                    let children_ranks = scan_result.children();
                    need_delete_ranks.extend(children_ranks);
                    for partial_overlap_rank in scan_result.partial_overlaps() {
                        if let Some(base_flex_id) = self.as_flex_id(&partial_overlap_rank) {
                            let diff = base_flex_id.difference(&flex_id);
                            need_delete_ranks.push(partial_overlap_rank);
                            need_insert_flex_ids.extend(diff);
                        }
                    }
                }
            }
            for rank in need_delete_ranks {
                self.remove_from_rank(rank);
            }
            for insert_id in need_insert_flex_ids {
                unsafe { self.join_uncheck_insert(insert_id) };
            }
        }
    }

    ///このSetが持っているFlexIdの個数を調べる
    pub fn size(&self) -> usize {
        self.main.len()
    }

    ///指定した領域を取得してSetを返す
    pub fn get<T: FlexIds>(&self, target: &T) -> Self {
        let ref_self = &*self;
        let scanner = ref_self.flex_id_scan_plan(target.clone());
        let mut result = Self::new();
        for flex_id_scanner in scanner.scan() {
            //もし、親に包まれていた場合はそのほかパターンを考える必要がない
            if flex_id_scanner.parent().is_some() {
                unsafe { result.join_uncheck_insert(flex_id_scanner.flex_id()) };
                continue;
            }

            //子を全て追加する
            for child_rank in flex_id_scanner.children() {
                let flex_id = self.as_flex_id(&child_rank).unwrap();
                unsafe { result.join_uncheck_insert(flex_id.clone()) };
            }

            //partial_overlapの重なりがある部分を全て追加する
            for partial_overlap_rank in flex_id_scanner.partial_overlaps() {
                let overlap_flex_id = self.as_flex_id(&partial_overlap_rank).unwrap();
                let intersection = overlap_flex_id
                    .intersection(&flex_id_scanner.flex_id())
                    .unwrap();
                unsafe { result.join_uncheck_insert(intersection) };
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

    ///Rankをreturnするためのメソット
    fn return_rank(&mut self, rank: u64) {
        if self.recycle_rank.len() < Self::RECYCLE_RANK_MAX {
            self.recycle_rank.push(rank);
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
        self.return_rank(rank);
        flex_id
    }

    pub fn as_flex_id(&self, rank: &FlexIdRank) -> Option<&FlexId> {
        self.main.get(&rank)
    }

    pub fn as_flex_ids(&self) -> impl Iterator<Item = &FlexId> {
        self.main.iter().map(|(_, v)| v)
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

    pub fn join(&mut self, target: &Self) {
        for flex_id in target.as_flex_ids() {
            self.insert(flex_id);
        }
    }

    pub fn union(&self, target: &Self) -> Self {
        let mut result;
        if self.size() > target.size() {
            result = self.clone();
            for flex_id in target.as_flex_ids() {
                result.insert(flex_id);
            }
        } else {
            result = target.clone();
            for flex_id in self.as_flex_ids() {
                result.insert(flex_id);
            }
        }
        result
    }

    pub fn intersection(&self, target: &Self) -> Self {
        let mut result = Self::new();
        if self.size() > target.size() {
            for flex_id in target.as_flex_ids() {
                let intersect = self.get(flex_id);
                result.join(&intersect);
            }
        } else {
            for flex_id in self.as_flex_ids() {
                let intersect = target.get(flex_id);
                result.join(&intersect);
            }
        }
        result
    }

    pub fn difference(&self, target: &Self) -> Self {
        let mut result = self.clone();
        for flex_id in target.as_flex_ids() {
            result.remove(flex_id);
        }
        result
    }

    ///このSetに入っている[RangeId]を全て返す
    pub fn range_ids(&self) -> impl Iterator<Item = RangeId> {
        self.main.iter().map(|(_, flex_id)| flex_id.range_id())
    }

    ///このSetに入っている[SingleId]を全て返す
    pub fn single_ids(&self) -> impl Iterator<Item = SingleId> {
        self.range_ids()
            .flat_map(|f| f.single_ids().collect::<Vec<_>>())
    }

    ///このSetが空かどうかを判定する
    pub fn is_empty(&self) -> bool {
        self.main.is_empty()
    }

    /// このSetが持っている最も大きなズームレベル値を返す
    /// (全探索を行うため、要素数に比例した計算コストがかかります)
    pub fn max_z(&self) -> Option<u8> {
        self.f
            .keys()
            .map(|s| s.to_f().0)
            .chain(self.x.keys().map(|s| s.to_xy().0))
            .chain(self.y.keys().map(|s| s.to_xy().0))
            .max()
    }

    /// このSetが持っている最も小さなズームレベル値を返す
    /// (全探索を行うため、要素数に比例した計算コストがかかります)
    pub fn min_z(&self) -> Option<u8> {
        self.f
            .keys()
            .map(|s| s.to_f().0)
            .chain(self.x.keys().map(|s| s.to_xy().0))
            .chain(self.y.keys().map(|s| s.to_xy().0))
            .min()
    }

    ///集合同士が指し示す範囲が等しいかを検証する
    pub fn equal(&self, target: &Self) -> bool {
        if self.size() != target.size() {
            return false;
        }
        let diff1 = self.difference(target);
        if !diff1.is_empty() {
            return false;
        }
        let diff2 = target.difference(self);
        diff2.is_empty()
    }
}
