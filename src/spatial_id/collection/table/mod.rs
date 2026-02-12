use std::{
    collections::{BTreeMap, HashMap, btree_map::Entry},
    thread::panicking,
};

use roaring::RoaringTreemap;

use crate::{
    FlexId, FlexIdRank, Segment,
    spatial_id::{
        FlexIds,
        collection::{RECYCLE_RANK_MAX, ValueRank, scanner::Scanner},
    },
};

pub struct TableOnMemory<V: Ord> {
    f: BTreeMap<Segment, RoaringTreemap>,
    x: BTreeMap<Segment, RoaringTreemap>,
    y: BTreeMap<Segment, RoaringTreemap>,
    main: HashMap<FlexIdRank, (FlexId, ValueRank)>,
    next_rank: u64,
    recycle_rank: Vec<u64>,

    //Table特有の要素
    ///Value側の[RoaringTreemap]にはこのValueを持つ[FlexIdRank]が入っている
    dictionary: BTreeMap<V, (RoaringTreemap, ValueRank)>,
    reverse: HashMap<ValueRank, V>,
    value_next_rank: u64,
    value_recycle_rank: Vec<u64>,
}

impl<V> Scanner for TableOnMemory<V>
where
    V: Ord + Clone,
{
    fn f(&self) -> &BTreeMap<Segment, RoaringTreemap> {
        &self.f
    }

    fn x(&self) -> &BTreeMap<Segment, RoaringTreemap> {
        &self.x
    }

    fn y(&self) -> &BTreeMap<Segment, RoaringTreemap> {
        &self.y
    }
}

impl<V> TableOnMemory<V>
where
    V: Ord + Clone,
{
    ///初期化する
    pub fn new() -> Self {
        Self {
            f: BTreeMap::new(),
            x: BTreeMap::new(),
            y: BTreeMap::new(),
            main: HashMap::new(),
            next_rank: 0,
            recycle_rank: Vec::new(),
            dictionary: BTreeMap::new(),
            reverse: HashMap::new(),
            value_next_rank: 0,
            value_recycle_rank: Vec::new(),
        }
    }

    ///値を挿入する
    pub fn insert<T: FlexIds>(&mut self, target: &T, value: &V) {
        //Targetに対するスキャンをする
        let scanner = self.flex_id_scan_plan(target.clone());

        //削除が必要なIDを貯めて最後に削除する
        let mut need_delete_ranks = RoaringTreemap::new();

        //挿入が必要なIDとValueを貯めて最後に挿入する
        let mut need_insert: Vec<(FlexId, &V)> = Vec::new();

        //既に同じValueがあるかを検索する
        let exist_same_value = self.find_value(value);

        for flex_id_scanner in scanner.scan() {
            //もし、親に包まれていた場合はそのほかパターンを考える必要がない
            if let Some(parent_rank) = flex_id_scanner.parent() {
                //親が持つFlexIdとValue Rankを取得する
                let parent = self.main.get(&parent_rank).unwrap();

                //もしも親が同様のValueを持つ場合
                if let Some(value_rank) = exist_same_value {
                    if parent.1 == value_rank {
                        continue;
                    }
                }

                //もしも親が異なるValueを持つ場合

                //親を分割
                let parent_splited = parent.0.difference(&flex_id_scanner.flex_id());

                //分割後の親を挿入予定にする
                for splited in parent_splited {}

                //親を削除する
                need_delete_ranks.insert(parent_rank);
            }

            //必ず削除しなければならないIDを削除予定にする
            need_delete_ranks |= flex_id_scanner.children();

            //競合解消が必要なIDを列挙する
            let partial_overlaps = flex_id_scanner.partial_overlaps();
        }
    }

    pub unsafe fn insert_unchecked<T: FlexIds>(&mut self, target: T, value: &V) {
        //割り当てるValueRank
        let value_rank = match self.find_value(value) {
            Some(rank) => rank,
            None => self.fetch_value_rank(),
        };
        //各次元に挿入を行う関数
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

        //ループ後にdictionaryに入れる
        let mut flex_id_ranks = RoaringTreemap::new();
        for flex_id in target.flex_ids() {
            let rank = self.fetch_rank();

            //分離している次元を更新
            dimension_insert(&mut self.f, flex_id.as_f().clone(), rank);
            dimension_insert(&mut self.x, flex_id.as_x().clone(), rank);
            dimension_insert(&mut self.y, flex_id.as_y().clone(), rank);

            //mainに挿入
            self.main.insert(rank, (flex_id, value_rank));

            //dictionaryに入れる準備
            flex_id_ranks.insert(rank);
        }

        //dictionaryを更新
    }

    ///既存のValueがある場合は見つけてくる
    fn find_value(&self, value: &V) -> Option<ValueRank> {
        let value_rank = self.dictionary.get(&value)?.1;
        Some(value_rank)
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
        if self.recycle_rank.len() < RECYCLE_RANK_MAX {
            self.recycle_rank.push(rank);
        }
    }

    ///新しいRankを予約するためのメソット
    fn fetch_value_rank(&mut self) -> ValueRank {
        match self.value_recycle_rank.pop() {
            Some(v) => v,
            None => {
                let result = self.value_next_rank;
                self.value_next_rank = self.value_next_rank + 1;
                result
            }
        }
    }

    ///Rankをreturnするためのメソット
    fn return_value_rank(&mut self, rank: u64) {
        if self.value_recycle_rank.len() < RECYCLE_RANK_MAX {
            self.value_recycle_rank.push(rank);
        }
    }
}
