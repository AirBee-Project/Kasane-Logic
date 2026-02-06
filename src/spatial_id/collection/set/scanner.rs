use std::{
    cell::{OnceCell, RefCell},
    collections::BTreeMap,
    ops::RangeInclusive,
};

use roaring::RoaringTreemap;
use std::ops::Bound::Included;

use crate::{
    FlexIdRank, Segment,
    spatial_id::{FlexIds, collection::set::SetOnMemory, segment},
};

///あるセグメントの他のセグメントの関係を記録する型
pub struct SegmentNeighborhood<'a> {
    ///対象となるセグメント
    segment: Segment,

    ///対象セグメントの親セグメント
    parents: OnceCell<RoaringTreemap>,

    ///対象セグメントの子セグメント
    children: OnceCell<RoaringTreemap>,

    ///スキャン対象となるBTree
    btree: &'a BTreeMap<Segment, RoaringTreemap>,
}

impl<'a> SegmentNeighborhood<'a> {
    fn new(segment: Segment, btree: &'a BTreeMap<Segment, RoaringTreemap>) -> Self {
        Self {
            segment,
            parents: OnceCell::new(),
            children: OnceCell::new(),
            btree,
        }
    }

    ///このセグメントの親セグメントを持つFlexIdのRankを全て返す
    fn parents(&self) -> &RoaringTreemap {
        self.parents.get_or_init(|| {
            let mut result = RoaringTreemap::new();
            for parent_segment in self.segment.self_and_parents() {
                let _ = match self.btree.get(&parent_segment) {
                    Some(v) => result.append(v),
                    None => {
                        continue;
                    }
                };
            }
            result
        })
    }

    fn children(&self) -> &RoaringTreemap {
        self.children.get_or_init(|| {
            let mut result = RoaringTreemap::new();
            //子のSegmentの探索の最終地点
            let end = self.segment.descendant_range_end();
            //子を全て探索
            let range_scan = match end {
                Some(v) => self.btree.range(self.segment.clone()..v),
                None => self.btree.range(self.segment.clone()..),
            };
            //全てをResultに返す
            for (_, flex_id_ranks) in range_scan {
                result.append(flex_id_ranks);
            }
            result
        })
    }
}

///Setの中からあるFlexIDに関連するIDを効率的にスキャンする
pub struct FlexIdScanPlan<'a> {
    ///スキャン対象となるSet
    set: &'a SetOnMemory,

    ///各次元の情報
    f: Vec<SegmentNeighborhood<'a>>,
    x: Vec<SegmentNeighborhood<'a>>,
    y: Vec<SegmentNeighborhood<'a>>,
}

impl<'a> FlexIdScanPlan<'a> {
    ///新しくスキャナーを作成する
    pub fn new<T: FlexIds>(set: &'a SetOnMemory, target: T) -> Self {
        let segmentation = target.segmentation();
        Self {
            set,
            f: segmentation
                .f
                .into_iter()
                .map(|segment| SegmentNeighborhood::new(segment, set.f()))
                .collect(),
            x: segmentation
                .x
                .into_iter()
                .map(|segment| SegmentNeighborhood::new(segment, set.x()))
                .collect(),
            y: segmentation
                .y
                .into_iter()
                .map(|segment| SegmentNeighborhood::new(segment, set.y()))
                .collect(),
        }
    }

    ///個別のスキャンを開始する
    pub fn scan(&self) -> impl Iterator<Item = FlexIdScanner<'_>> {
        self.f.iter().flat_map(move |f| {
            self.x.iter().flat_map(move |x| {
                self.y.iter().map(move |y| FlexIdScanner {
                    set: self.set,
                    f,
                    x,
                    y,
                })
            })
        })
    }
}

pub struct FlexIdScanner<'a> {
    ///スキャン対象となるSet
    set: &'a SetOnMemory,

    ///各次元のデータを持っている
    f: &'a SegmentNeighborhood<'a>,
    x: &'a SegmentNeighborhood<'a>,
    y: &'a SegmentNeighborhood<'a>,

    //計算結果をキャッシュしておく
    parent: OnceCell<Option<FlexIdRank>>,
    children: OnceCell<RoaringTreemap>,
}

impl FlexIdScanner<'_> {
    ///親のFlexIdがあるかどうかを判定し、あればそのRankを返す
    /// Parentには自分と同じ形のFlexIdも含まれる
    /// ParentはSetの定義上、必ず0-1個である
    pub fn unique_parent(&self) -> Option<FlexIdRank> {
        *self.parent.get_or_init(|| {
            let f = self.f.parents();
            let x = self.x.parents();
            let y = self.y.parents();
            let intersection = f & x & y;

            #[cfg(debug_assertions)]
            if intersection.len() > 1 {
                panic!("親のIDが2つ以上検知されました")
            }
            intersection.iter().next()
        })
    }

    ///完全に自分含むFlexIdのRankを返す
    pub fn children(&self) -> RoaringTreemap {
        self.children
            .get_or_init(|| {
                let f = self.f.children();
                let x = self.x.children();
                let y = self.y.children();
                f & x & y
            })
            .clone()
    }

    ///親でも子でもなく、部分的に重なっているFlexIdのRankを返す
    pub fn partial_overlaps() -> RoaringTreemap {
        todo!()
    }
}
