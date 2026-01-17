use roaring::RoaringTreemap;
use std::collections::BTreeMap;

use crate::spatial_id::{SpatialId, encode::EncodeId, segment::encode::EncodeSegment};
use std::ops::Bound::Excluded;

type Rank = u64;

pub struct SpatialIdMap {
    f: BTreeMap<EncodeSegment, RoaringTreemap>,
    x: BTreeMap<EncodeSegment, RoaringTreemap>,
    y: BTreeMap<EncodeSegment, RoaringTreemap>,
    main: BTreeMap<Rank, EncodeId>,
    next_rank: Rank,
}

impl SpatialIdMap {
    pub fn new() -> Self {
        Self {
            f: BTreeMap::new(),
            x: BTreeMap::new(),
            y: BTreeMap::new(),
            main: BTreeMap::new(),
            next_rank: 0,
        }
    }

    pub fn insert<T: SpatialId>(spatial_id: T) {}

    ///Setの中から関連のあるIDを見つける
    /// Setの中から関連のあるIDを見つける
    pub fn find_related_id(&self, encode_id: EncodeId) -> Vec<Rank> {
        // ヘルパー: ある次元のマップとターゲットセグメントを受け取り、関連するRankの集合を返す
        let get_related_ranks = |map: &BTreeMap<EncodeSegment, RoaringTreemap>,
                                 target: &EncodeSegment|
         -> RoaringTreemap {
            let mut related_bitmap = RoaringTreemap::new();

            // 1. 等価 (Equal) および 上位セグメント (Ancestor) の検索
            // ターゲットから親へ向かってルートまで遡る
            let mut current = Some(target.clone());
            while let Some(seg) = current {
                if let Some(ranks) = map.get(&seg) {
                    // 和集合 (OR) をとる
                    related_bitmap |= ranks;
                }
                current = seg.parent();
            }

            // 2. 下位セグメント (Descendant) の検索
            // BTreeMapの範囲検索機能を使用
            // 範囲: (ターゲット(除外) 〜 ターゲットの子孫の終わり(除外))
            // ※ターゲット自体はStep 1で取得済みなのでExcluded
            let range_end = target.children_range_end();
            for (_, ranks) in map.range((Excluded(target), Excluded(&range_end))) {
                // 和集合 (OR) をとる
                related_bitmap |= ranks;
            }

            related_bitmap
        };

        // 各次元について関連するID集合を取得
        let f_related = get_related_ranks(&self.f, encode_id.as_f());
        let x_related = get_related_ranks(&self.x, encode_id.as_x());
        let y_related = get_related_ranks(&self.y, encode_id.as_y());

        // 3. 全次元の積集合 (Intersection / AND) をとる
        // これが「関連のある符号化空間ID」の定義（全ての次元で関連していること）を満たす
        // RoaringTreemap同士の & 演算は非常に高速
        let result_bitmap = f_related & x_related & y_related;

        // Vec<Rank> に変換して返す
        result_bitmap.iter().collect()
    }

    ///Setの中から特定のIDを削除する
    fn delete_id(rank: Rank) {}
}
