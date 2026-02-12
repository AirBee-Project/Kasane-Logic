use std::{
    collections::{BTreeMap, HashMap, btree_map::Entry},
    hash::Hash,
    thread::panicking,
};

use roaring::RoaringTreemap;

use crate::{
    FlexId, FlexIdRank, RangeId, Segment,
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
    V: Ord + Clone + Hash,
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
        let mut need_insert: HashMap<V, Vec<FlexId>> = HashMap::new();

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
                else {
                    //親を分割
                    let parent_splited = parent.0.difference(&flex_id_scanner.flex_id());

                    //親の持っていたValueを取得
                    let parent_value = self.reverse.get(&parent.1).unwrap();

                    //分割後の親を挿入予定にする
                    for splited in parent_splited {
                        match need_insert.entry(parent_value.clone()) {
                            std::collections::hash_map::Entry::Occupied(mut occupied_entry) => {
                                let exist = occupied_entry.get_mut();
                                exist.push(splited);
                            }
                            std::collections::hash_map::Entry::Vacant(vacant_entry) => {
                                vacant_entry.insert(vec![splited]);
                            }
                        }
                    }

                    //親を削除予定にする
                    need_delete_ranks.insert(parent_rank);
                }

                //親を処理すれば終わり
                continue;
            }

            //子は必ず削除しなければならないIDを削除予定にする
            need_delete_ranks |= flex_id_scanner.children();

            //競合解消が必要なIDを列挙する
            let partial_overlaps = flex_id_scanner.partial_overlaps();

            //競合解消が不要な場合は次のFLexIDへ行く
            if partial_overlaps.is_empty() {
                match need_insert.entry(value.clone()) {
                    std::collections::hash_map::Entry::Occupied(mut occupied_entry) => {
                        let exist = occupied_entry.get_mut();
                        exist.push(flex_id_scanner.flex_id().clone());
                    }
                    std::collections::hash_map::Entry::Vacant(vacant_entry) => {
                        vacant_entry.insert(vec![flex_id_scanner.flex_id().clone()]);
                    }
                }
                continue;
            }

            //もともとの競合IDは削除予定にする
            need_delete_ranks |= flex_id_scanner.partial_overlaps();

            //競合解消が必要なIDをひたすら削っていく
            //Setとは違って、相手を削る
            for partial_overlap_rank in partial_overlaps {
                let partial_overlap = self.main.get(&partial_overlap_rank).unwrap();

                let overlap_splited = partial_overlap.0.difference(&flex_id_scanner.flex_id());

                for splited in overlap_splited {
                    match need_insert.entry(value.clone()) {
                        std::collections::hash_map::Entry::Occupied(mut occupied_entry) => {
                            let exist = occupied_entry.get_mut();
                            exist.push(splited);
                        }
                        std::collections::hash_map::Entry::Vacant(vacant_entry) => {
                            vacant_entry.insert(vec![splited]);
                        }
                    }
                }
            }
        }

        //削除が必要なものを削除する
        for nend_delete_rank in need_delete_ranks {
            self.remove_from_rank(nend_delete_rank);
        }

        //挿入が必要なものを挿入する
        for (value, flex_ids) in need_insert {
            for flex_id in flex_ids {
                unsafe { self.join_insert_unchecked(flex_id, &value) };
            }
        }
    }

    pub fn range_ids(&self) -> impl Iterator<Item = (RangeId, &V)> {
        self.main.iter().map(|(_, (flex_id, value_rank))| {
            let range_id = flex_id.range_id();
            let value = self.reverse.get(value_rank).unwrap();
            (range_id, value)
        })
    }

    pub unsafe fn join_insert_unchecked<T: FlexIds>(&mut self, target: T, value: &V) {
        match self.find_value(value) {
            //既に値が存在するので、探索する価値がある
            Some(_) => {
                for flex_id in target.flex_ids() {
                    match self.find(flex_id.f_sibling()) {
                        Some(v) => match flex_id.f_parent() {
                            Some(parent) => {
                                let sibling_value_rank = self.main.get(&v).unwrap().1;
                                let sibling_value = self.reverse.get(&sibling_value_rank).unwrap();
                                if sibling_value == value {
                                    self.remove_from_rank(v);
                                    unsafe { self.join_insert_unchecked(parent, value) };
                                    continue;
                                }
                            }
                            None => {}
                        },
                        None => {}
                    }

                    // X軸方向の結合チェック
                    match self.find(flex_id.x_sibling()) {
                        Some(v) => {
                            let sibling_value_rank = self.main.get(&v).unwrap().1;
                            let sibling_value = self.reverse.get(&sibling_value_rank).unwrap();
                            if sibling_value == value {
                                self.remove_from_rank(v);
                                unsafe {
                                    self.join_insert_unchecked(flex_id.x_parent().unwrap(), value)
                                };
                                continue;
                            }
                        }
                        None => {}
                    }

                    // Y軸方向の結合チェック
                    match self.find(flex_id.y_sibling()) {
                        Some(v) => {
                            let sibling_value_rank = self.main.get(&v).unwrap().1;
                            let sibling_value = self.reverse.get(&sibling_value_rank).unwrap();
                            if sibling_value == value {
                                self.remove_from_rank(v);
                                unsafe {
                                    self.join_insert_unchecked(flex_id.y_parent().unwrap(), value)
                                };
                                continue;
                            }
                        }
                        None => {}
                    }

                    // 結合相手がいなければ、そのまま挿入
                    unsafe { self.insert_unchecked(flex_id, value) };
                }
            }
            //同じValueがないので、無条件挿入
            None => {
                unsafe { self.insert_unchecked(target.clone(), value) };
            }
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

        //空間IDの情報を更新
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

        //dictonaryの更新
        match self.dictionary.entry(value.clone()) {
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert((flex_id_ranks, value_rank));
                //逆引きの更新
                self.reverse.insert(value_rank, value.clone());
            }
            //既に存在する場合はそのValueを持つFlexIdを更新
            Entry::Occupied(mut occupied_entry) => {
                let exist = occupied_entry.get_mut();
                exist.0 |= flex_id_ranks
            }
        }
    }

    ///既存のValueがある場合は見つけてくる
    fn find_value(&self, value: &V) -> Option<ValueRank> {
        let value_rank = self.dictionary.get(&value)?.1;
        Some(value_rank)
    }

    ///Rankを指定して削除する
    ///存在しないRankをリクエストされた場合はPanicします
    fn remove_from_rank(&mut self, rank: FlexIdRank) -> (FlexId, V) {
        //特定の次元から削除する
        let dimension_remove =
            |btree: &mut BTreeMap<Segment, RoaringTreemap>, segment: &Segment, rank: FlexIdRank| {
                if let Some(entry) = btree.get_mut(segment) {
                    entry.remove(rank);
                    if entry.is_empty() {
                        btree.remove(segment);
                    }
                }
            };

        //存在しないRankをリクエストされた場合はPanicします。
        let flex_id = self.main.remove(&rank).unwrap();
        dimension_remove(&mut self.f, flex_id.0.as_f(), rank);
        dimension_remove(&mut self.x, flex_id.0.as_x(), rank);
        dimension_remove(&mut self.y, flex_id.0.as_y(), rank);
        self.return_rank(rank);

        //Value部分を削除する
        let value = self.reverse.get(&flex_id.1).unwrap().clone();
        let dictionary = self.dictionary.get_mut(&value).unwrap();
        let value_rank = dictionary.1.clone();
        dictionary.0.remove(rank);

        //もしそのValueを持つ[FlexId]が存在しない場合はKeyごと削除する
        if dictionary.0.is_empty() {
            self.dictionary.remove(&value);
            self.reverse.remove(&value_rank);
        }

        //recycleに使用されていたvalue-rankを返す
        self.return_value_rank(value_rank);

        return (flex_id.0, value);
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
