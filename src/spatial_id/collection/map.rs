use roaring::RoaringTreemap;
use std::collections::BTreeMap;

use crate::spatial_id::{
    SpatialIdEncode,
    collection::{map, set::SpatialIdSet},
    encode::EncodeId,
    range::RangeId,
    segment::encode::EncodeSegment,
};
use std::ops::Bound::{Excluded, Included};

type Rank = u64;

#[derive(Clone)]
pub struct SpatialIdMap<V> {
    pub(crate) f: BTreeMap<EncodeSegment, RoaringTreemap>,
    pub(crate) x: BTreeMap<EncodeSegment, RoaringTreemap>,
    pub(crate) y: BTreeMap<EncodeSegment, RoaringTreemap>,
    pub(crate) main: BTreeMap<Rank, (EncodeId, V)>,
    pub(crate) next_rank: Rank,
}

impl<V> SpatialIdMap<V>
where
    V: Clone + PartialEq,
{
    ///新しく[SpatialIdMap]を作成する。
    pub fn new() -> Self {
        Self {
            f: BTreeMap::new(),
            x: BTreeMap::new(),
            y: BTreeMap::new(),
            main: BTreeMap::new(),
            next_rank: 0,
        }
    }

    /// Mapの値を捨てて、領域情報のみを持つSetに変換する
    pub fn to_set(self) -> SpatialIdSet {
        let mut set = SpatialIdSet::new();
        for (encode_id, _) in self.main.into_values() {
            // Setのinsertは値を無視して結合を行うため、
            // Map時代には色が違って分かれていたIDもここで結合される
            set.insert(&encode_id);
        }
        set
    }

    ///最適化された結合
    pub fn to_set_join(self) -> SpatialIdSet {
        let mut set = SpatialIdSet::new();
        for (encode_id, _) in self.main.into_values() {
            // Setのinsertは値を無視して結合を行うため、
            // Map時代には色が違って分かれていたIDもここで結合される
            set.insert(&encode_id);
        }
        set
    }

    /// キー（RangeId）のイテレータ
    pub fn keys(&self) -> impl Iterator<Item = RangeId> + '_ {
        self.main.values().map(|(encode_id, _)| encode_id.decode())
    }

    /// キー（RangeId）のイテレータ
    pub fn keys_encode(&self) -> impl Iterator<Item = EncodeId> + '_ {
        self.main.values().map(|(encode_id, _)| encode_id.clone())
    }

    /// 値のイテレータ
    pub fn values(&self) -> impl Iterator<Item = &V> + '_ {
        self.main.values().map(|(_, v)| v)
    }

    /// キーと値のイテレータ
    pub fn iter(&self) -> impl Iterator<Item = (RangeId, &V)> + '_ {
        self.main
            .values()
            .map(|(encode_id, v)| (encode_id.decode(), v))
    }

    ///[SpatialIdMap]に入っている[EncodeId]の個数を返す。
    pub fn size(&self) -> usize {
        self.main.len()
    }

    ///[SpatialIdMap]の中にある`target`と関連ある[EncodeId]のRankを返す。
    pub(crate) fn related(&self, target: &EncodeId) -> RoaringTreemap {
        let related_segments = |map: &BTreeMap<EncodeSegment, RoaringTreemap>,
                                target_seg: &EncodeSegment|
         -> RoaringTreemap {
            let mut related_bitmap = RoaringTreemap::new();

            let mut current = target_seg.parent();
            while let Some(seg) = current {
                if let Some(ranks) = map.get(&seg) {
                    related_bitmap |= ranks;
                }
                current = seg.parent();
            }

            let range_end = target_seg.descendant_range_end();
            for (_, ranks) in map.range((Included(target_seg), Excluded(&range_end))) {
                related_bitmap |= ranks;
            }

            related_bitmap
        };

        let f_related = related_segments(&self.f, target.as_f());
        let x_related = related_segments(&self.x, target.as_x());
        let y_related = related_segments(&self.y, target.as_y());

        // 3次元すべての積集合をとる
        f_related & x_related & y_related
    }

    pub unsafe fn insert_unchecked<T: SpatialIdEncode>(&mut self, target: &T, value: &V) {
        for encode_id in target.encode() {
            let rank = self.next_rank;
            self.next_rank += 1;

            self.f
                .entry(encode_id.as_f().clone())
                .or_default()
                .insert(rank);
            self.x
                .entry(encode_id.as_x().clone())
                .or_default()
                .insert(rank);
            self.y
                .entry(encode_id.as_y().clone())
                .or_default()
                .insert(rank);
            self.main.insert(rank, (encode_id.clone(), value.clone()));
        }
    }

    ///[SpatialIdMap]に[SpatialId]を挿入する。
    pub fn insert<T: SpatialIdEncode>(&mut self, target: &T, value: &V) {
        for encode_id in target.encode() {
            let related_ranks = self.related(&encode_id);

            // 処理が必要な既存IDのリスト
            // (Rank, Case)
            // Case A: 既存IDを削除する (新しいIDが飲み込む)
            // Case B: 既存IDを分割する (新しいIDが内部を上書きする)
            let mut to_process = Vec::new();
            let mut is_completely_covered = false;

            for rank in related_ranks {
                if let Some((existing_id, existing_val)) = self.main.get(&rank) {
                    if existing_id.contains(&encode_id) {
                        // Case: 既存ID(大) >= 新ID(小)
                        if existing_val == value {
                            // 値も同じなら、何もしなくて良い（既にその値でカバーされている）
                            is_completely_covered = true;
                            break;
                        } else {
                            // 値が違うなら、既存IDを「ドーナツ状」に分割して再登録し、
                            // 中央（新ID部分）は後で insert する
                            to_process.push((rank, "SPLIT"));
                        }
                    } else if encode_id.contains(existing_id) {
                        // Case: 新ID(大) >= 既存ID(小)
                        // 既存IDは完全に上書きされるので削除
                        to_process.push((rank, "REMOVE"));
                    }
                }
            }

            if is_completely_covered {
                continue;
            }

            for (rank, action) in to_process {
                // まず既存を削除して取り出す
                if let Some((old_id, old_val)) = self.remove_rank(rank) {
                    if action == "SPLIT" {
                        // 差分計算: (既存) - (新)
                        // これにより「新IDと被らない部分」だけが「元の値」で残る
                        let diff = old_id.difference(&encode_id);
                        for piece in diff {
                            unsafe { self.join_insert_unchecked(&piece, &old_val) };
                        }
                    }
                    // REMOVEの場合は削除したままでOK
                }
            }

            // 最後に新しいIDを挿入
            unsafe { self.join_insert_unchecked(&encode_id, value) };
        }
    }

    ///対象のID領域を削除する。
    pub fn remove<T: SpatialIdEncode>(&mut self, target: &T) {
        for encode_id in target.encode() {
            for related_rank in self.related(&encode_id) {
                let base = self.main.remove(&related_rank).unwrap();
                self.f.remove(base.0.as_f());
                self.x.remove(base.0.as_x());
                self.y.remove(base.0.as_y());
                let diff = base.0.difference(&encode_id);
                for need_insert in diff {
                    unsafe { self.join_insert_unchecked(&need_insert, &base.1) }
                }
            }
        }
    }

    ///[SpatialIdMap]の特定の[SpatialId]と重なる部分だけを取り出す。
    pub fn subset<T: SpatialIdEncode>(&self, target: &T) -> SpatialIdMap<V> {
        let mut result = SpatialIdMap::new();
        for encode_id in target.encode() {
            for related_rank in self.related(&encode_id) {
                let base = self.main.get(&related_rank).unwrap();
                let intersection = encode_id.intersection(&base.0).unwrap();
                unsafe { result.join_insert_unchecked(&intersection, &base.1) };
            }
        }
        result
    }

    /// IDを追加し、可能な場合は結合を行う。
    /// 重複チェックは行っていないので、その責任は関数の使用者が負う。
    pub unsafe fn join_insert_unchecked(&mut self, target: &EncodeId, value: &V) {
        // --- F次元 ---
        // sibling() が Option を返す前提で記述 (前回の議論を反映)
        let f_sibling = EncodeId::new(
            target.as_f().sibling(),
            target.as_x().clone(),
            target.as_y().clone(),
        );

        if let Some(rank) = self.find_encode(&f_sibling) {
            // 【重要】値のチェックを追加
            // 兄弟が存在し、かつ「値が同じ」なら結合する
            if let Some((_, v)) = self.main.get(&rank) {
                if v == value {
                    self.remove_rank(rank);
                    // 再帰的に親で試行
                    unsafe {
                        self.join_insert_unchecked(
                            &EncodeId::new(
                                target.as_f().parent().unwrap(),
                                target.as_x().clone(),
                                target.as_y().clone(),
                            ),
                            value,
                        )
                    };
                    return;
                }
            }
        }

        // --- X次元 ---
        let x_sibling = EncodeId::new(
            target.as_f().clone(),
            target.as_x().sibling(),
            target.as_y().clone(),
        );

        if let Some(rank) = self.find_encode(&x_sibling) {
            if let Some((_, v)) = self.main.get(&rank) {
                if v == value {
                    self.remove_rank(rank);
                    unsafe {
                        self.join_insert_unchecked(
                            &EncodeId::new(
                                target.as_f().clone(),
                                target.as_x().parent().unwrap(),
                                target.as_y().clone(),
                            ),
                            value,
                        )
                    };
                    return;
                }
            }
        }

        // --- Y次元 ---
        let y_sibling = EncodeId::new(
            target.as_f().clone(),
            target.as_x().clone(),
            target.as_y().sibling(),
        );

        if let Some(rank) = self.find_encode(&y_sibling) {
            if let Some((_, v)) = self.main.get(&rank) {
                if v == value {
                    self.remove_rank(rank);
                    unsafe {
                        self.join_insert_unchecked(
                            &EncodeId::new(
                                target.as_f().clone(),
                                target.as_x().clone(),
                                target.as_y().parent().unwrap(),
                            ),
                            value,
                        )
                    };
                    return;
                }
            }
        }

        // 結合できなければそのまま挿入
        unsafe { self.insert_unchecked(target, value) };
    }

    ///指定されたEncodeIdと完全に一致するEncodeIdのRankを返す。
    fn find_encode(&self, target: &EncodeId) -> Option<Rank> {
        let f_hits = self.f.get(target.as_f())?;
        let x_hits = self.x.get(target.as_x())?;
        let y_hits = self.y.get(target.as_y())?;
        let result = f_hits & x_hits & y_hits;
        result.iter().next()
    }

    /// 指定されたRankを持つIDを全てのインデックスから完全に削除する。
    fn remove_rank(&mut self, rank: Rank) -> Option<(EncodeId, V)> {
        let (encode_id, val) = self.main.remove(&rank)?;

        let remove_from_dim = |map: &mut BTreeMap<EncodeSegment, RoaringTreemap>,
                               segment: EncodeSegment| {
            if let std::collections::btree_map::Entry::Occupied(mut entry) = map.entry(segment) {
                let bitmap = entry.get_mut();
                bitmap.remove(rank);
                if bitmap.is_empty() {
                    entry.remove_entry();
                }
            }
        };

        remove_from_dim(&mut self.f, encode_id.as_f().clone());
        remove_from_dim(&mut self.x, encode_id.as_x().clone());
        remove_from_dim(&mut self.y, encode_id.as_y().clone());

        Some((encode_id, val))
    }

    /// 空かどうか
    pub fn is_empty(&self) -> bool {
        self.main.is_empty()
    }

    /// 全削除
    pub fn clear(&mut self) {
        self.f.clear();
        self.x.clear();
        self.y.clear();
        self.main.clear();
        self.next_rank = 0;
    }
}
