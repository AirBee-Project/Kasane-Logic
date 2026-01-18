use crate::{
    kv::KvStore,
    spatial_id::{
        SpatialIdEncode,
        collection::{MapTrait, Rank},
        encode::EncodeId,
        range::RangeId,
        segment::encode::EncodeSegment,
    },
};
use roaring::RoaringTreemap;
use std::cell::Cell;
use std::collections::BTreeMap;
use std::ops::Bound::{Excluded, Included};

#[derive(Clone)]
pub struct Map<V> {
    f: BTreeMap<EncodeSegment, RoaringTreemap>,
    x: BTreeMap<EncodeSegment, RoaringTreemap>,
    y: BTreeMap<EncodeSegment, RoaringTreemap>,
    main: BTreeMap<Rank, (EncodeId, V)>,
    next_rank: Cell<Rank>,
}

impl<V> Map<V> {
    pub fn new() -> Self {
        Self {
            f: BTreeMap::new(),
            x: BTreeMap::new(),
            y: BTreeMap::new(),
            main: BTreeMap::new(),
            next_rank: Cell::new(0),
        }
    }
}

impl<V> Default for Map<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> MapTrait for Map<V> {
    type V = V;

    type DimensionMap = BTreeMap<EncodeSegment, RoaringTreemap>;
    type MainMap = BTreeMap<Rank, (EncodeId, V)>;

    fn f(&self) -> &Self::DimensionMap {
        &self.f
    }

    fn f_mut(&mut self) -> &mut Self::DimensionMap {
        &mut self.f
    }

    fn x(&self) -> &Self::DimensionMap {
        &self.x
    }

    fn x_mut(&mut self) -> &mut Self::DimensionMap {
        &mut self.x
    }

    fn y(&self) -> &Self::DimensionMap {
        &self.y
    }

    fn y_mut(&mut self) -> &mut Self::DimensionMap {
        &mut self.y
    }

    fn main(&self) -> &Self::MainMap {
        &self.main
    }

    fn main_mut(&mut self) -> &mut Self::MainMap {
        &mut self.main
    }

    fn fetch_next_rank(&self) -> Rank {
        let rank = self.next_rank.get();
        self.next_rank.set(rank + 1);
        rank
    }

    fn clear(&mut self) {
        self.f.clear();
        self.x.clear();
        self.y.clear();
        self.main.clear();
        self.next_rank.set(0);
    }
}

#[derive(Clone)]
pub struct MapLogic<S>(pub S);

impl<S> MapLogic<S>
where
    S: MapTrait + Default,
    S::V: Clone + PartialEq,
{
    /// 新しいマップを作成（ストアを受け取る）
    pub fn new(store: S) -> Self {
        Self(store)
    }

    /// 内部のストアへの参照を取得
    pub(crate) fn inner(&self) -> &S {
        &self.0
    }

    /// 内部のストアへの可変参照を取得
    pub(crate) fn inner_mut(&mut self) -> &mut S {
        &mut self.0
    }

    pub fn keys(&self) -> impl Iterator<Item = RangeId> + '_ {
        self.0.main().iter().map(|(_, (id, _))| id.decode())
    }

    pub fn keys_encode(&self) -> impl Iterator<Item = EncodeId> + '_ {
        self.0.main().iter().map(|(_, (id, _))| id.clone())
    }

    pub fn values(&self) -> impl Iterator<Item = &S::V> + '_ {
        self.0.main().iter().map(|(_, (_, v))| v)
    }

    pub fn iter(&self) -> impl Iterator<Item = (RangeId, &S::V)> + '_ {
        self.0.main().iter().map(|(_, (id, v))| (id.decode(), v))
    }

    pub fn size(&self) -> usize {
        self.0.main().len()
    }

    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    // ---------------------------------------------------------------------
    //  Core Logic
    // ---------------------------------------------------------------------

    pub(crate) fn related(&self, target: &EncodeId) -> RoaringTreemap {
        // ヘルパー: 特定の次元マップから関連Rankを取得
        let get_related = |map: &S::DimensionMap, seg: &EncodeSegment| -> RoaringTreemap {
            let mut bitmap = RoaringTreemap::new();

            // Ancestors (親を辿る)
            let mut current = seg.parent();
            while let Some(parent) = current {
                if let Some(ranks) = map.get(&parent) {
                    bitmap |= ranks;
                }
                current = parent.parent();
            }

            // Descendants & Self (範囲検索)
            let end = seg.descendant_range_end();
            for (_, ranks) in map.range((Included(seg), Excluded(&end))) {
                bitmap |= ranks;
            }
            bitmap
        };

        let f_related = get_related(self.0.f(), target.as_f());
        let x_related = get_related(self.0.x(), target.as_x());
        let y_related = get_related(self.0.y(), target.as_y());

        f_related & x_related & y_related
    }

    fn find_encode(&self, target: &EncodeId) -> Option<Rank> {
        let f_hits = self.0.f().get(target.as_f())?;
        let x_hits = self.0.x().get(target.as_x())?;
        let y_hits = self.0.y().get(target.as_y())?;

        (f_hits & x_hits & y_hits).iter().next()
    }

    /// 要素を挿入（競合解決・分割処理付き）
    pub fn insert<T: SpatialIdEncode>(&mut self, target: &T, value: &S::V) {
        for encode_id in target.encode() {
            let related_ranks = self.related(&encode_id);

            let mut to_process = Vec::new();
            let mut is_completely_covered = false;

            for rank in related_ranks {
                if let Some((existing_id, existing_val)) = self.0.main().get(&rank) {
                    if existing_id.contains(&encode_id) {
                        // Case: 既存(大) >= 新(小)
                        if existing_val == value {
                            is_completely_covered = true;
                            break;
                        } else {
                            // 値が違うなら分割のために一時保存
                            to_process.push((rank, "SPLIT"));
                        }
                    } else if encode_id.contains(existing_id) {
                        // Case: 新(大) >= 既存(小)
                        to_process.push((rank, "REMOVE"));
                    }
                }
            }

            if is_completely_covered {
                continue;
            }

            for (rank, action) in to_process {
                // 削除して値を取り出す
                if let Some((old_id, old_val)) = self.remove_rank(rank) {
                    if action == "SPLIT" {
                        let diff = old_id.difference(&encode_id);
                        for piece in diff {
                            unsafe { self.join_insert_unchecked(&piece, &old_val) };
                        }
                    }
                }
            }

            unsafe { self.join_insert_unchecked(&encode_id, value) };
        }
    }

    /// 要素を削除
    pub fn remove<T: SpatialIdEncode>(&mut self, target: &T) {
        for encode_id in target.encode() {
            let related_ranks = self.related(&encode_id);
            // 削除対象リストを作成（イテレータの借用問題を回避）
            let ranks: Vec<Rank> = related_ranks.into_iter().collect();

            for rank in ranks {
                if let Some((existing_id, existing_val)) = self.remove_rank(rank) {
                    let diff = existing_id.difference(&encode_id);
                    for piece in diff {
                        unsafe { self.join_insert_unchecked(&piece, &existing_val) };
                    }
                }
            }
        }
    }

    /// 部分集合の取得
    pub fn subset<T: SpatialIdEncode>(&self, target: &T) -> MapLogic<S> {
        // 同じストレージ型 S を使って新しい空のMapを作成
        // (S: Default が必要)
        let mut result_map = MapLogic::new(S::default());

        for encode_id in target.encode() {
            let related_ranks = self.related(&encode_id);
            for rank in related_ranks {
                // 元のMapから値も取得する
                if let Some((existing_id, existing_val)) = self.0.main().get(&rank) {
                    if let Some(intersection) = encode_id.intersection(existing_id) {
                        // 値 (existing_val) も一緒に新しいMapに挿入
                        unsafe { result_map.join_insert_unchecked(&intersection, existing_val) };
                    }
                }
            }
        }
        result_map
    }

    /// 結合付き挿入（内部用）
    pub unsafe fn join_insert_unchecked(&mut self, target: &EncodeId, value: &S::V) {
        // --- F次元の結合チェック ---

        let f_sibling = EncodeId::new(
            target.as_f().sibling(),
            target.as_x().clone(),
            target.as_y().clone(),
        );
        if let Some(rank) = self.find_encode(&f_sibling) {
            if let Some((_, v)) = self.0.main().get(&rank) {
                if v == value {
                    self.remove_rank(rank);
                    let parent = target.as_f().parent().unwrap();
                    unsafe {
                        self.join_insert_unchecked(
                            &EncodeId::new(parent, target.as_x().clone(), target.as_y().clone()),
                            value,
                        )
                    };
                    return;
                }
            }
        }

        // --- X次元の結合チェック ---
        let x_sibling = EncodeId::new(
            target.as_f().clone(),
            target.as_x().sibling(),
            target.as_y().clone(),
        );
        if let Some(rank) = self.find_encode(&x_sibling) {
            if let Some((_, v)) = self.0.main().get(&rank) {
                if v == value {
                    self.remove_rank(rank);
                    let parent = target.as_x().parent().unwrap();
                    unsafe {
                        self.join_insert_unchecked(
                            &EncodeId::new(target.as_f().clone(), parent, target.as_y().clone()),
                            value,
                        )
                    };
                    return;
                }
            }
        }

        // --- Y次元の結合チェック ---
        let y_sibling = EncodeId::new(
            target.as_f().clone(),
            target.as_x().clone(),
            target.as_y().sibling(),
        );
        if let Some(rank) = self.find_encode(&y_sibling) {
            if let Some((_, v)) = self.0.main().get(&rank) {
                if v == value {
                    self.remove_rank(rank);
                    let parent = target.as_y().parent().unwrap();
                    unsafe {
                        self.join_insert_unchecked(
                            &EncodeId::new(target.as_f().clone(), target.as_x().clone(), parent),
                            value,
                        )
                    };
                    return;
                }
            }
        }

        // 結合できなければ挿入
        unsafe { self.insert_unchecked(target, value) };
    }

    /// 生の挿入処理 (KvStoreへの書き込み)
    pub unsafe fn insert_unchecked(&mut self, target: &EncodeId, value: &S::V) {
        let rank = self.0.fetch_next_rank();

        // ヘルパー: Bitmapへの追加 (Entry APIの代用)
        let upsert = |map: &mut S::DimensionMap, key: &EncodeSegment| {
            // updateができれば使う、できなければ get -> insert/new
            let mut done = false;
            map.update(key, |bm| {
                bm.insert(rank);
                done = true;
            });
            if !done {
                let mut bm = RoaringTreemap::new();
                bm.insert(rank);
                map.insert(key.clone(), bm);
            }
        };

        upsert(self.0.f_mut(), target.as_f());
        upsert(self.0.x_mut(), target.as_x());
        upsert(self.0.y_mut(), target.as_y());

        self.0
            .main_mut()
            .insert(rank, (target.clone(), value.clone()));
    }

    /// 指定Rankの削除
    fn remove_rank(&mut self, rank: Rank) -> Option<(EncodeId, S::V)> {
        // Mainから削除してIDと値を取得
        let (encode_id, val) = self.0.main_mut().remove(&rank)?;

        // 各次元インデックスからRankを削除
        let remove_from_dim = |map: &mut S::DimensionMap, key: &EncodeSegment| {
            let mut should_remove_key = false;
            map.update(key, |bm| {
                bm.remove(rank);
                if bm.is_empty() {
                    should_remove_key = true;
                }
            });
            // 空になったキーは削除（GC）
            if should_remove_key {
                map.remove(key);
            }
        };

        remove_from_dim(self.0.f_mut(), encode_id.as_f());
        remove_from_dim(self.0.x_mut(), encode_id.as_x());
        remove_from_dim(self.0.y_mut(), encode_id.as_y());

        Some((encode_id, val))
    }
}
