use std::{cell::OnceCell, collections::BTreeMap};

use crate::{
    FlexId, FlexIdRank, FlexIdRankList, Segment, SpatialId,
    spatial_id::collection::v_bit::core::{VBitCore, flex_id_rank_list::FlexIdRankListMultiOps},
};

#[derive(Debug)]
///あるセグメントの他のセグメントの関係を記録する型
pub struct SegmentNeighborhood<'a> {
    ///対象となるセグメント
    segment: Segment<8>,

    ///対象セグメントの親セグメント
    parents: OnceCell<FlexIdRankList>,

    ///対象セグメントの子セグメント
    children: OnceCell<FlexIdRankList>,

    ///スキャン対象となるBTree
    btree: &'a BTreeMap<Segment<8>, FlexIdRankList>,
}

impl<'a> SegmentNeighborhood<'a> {
    fn new(segment: Segment<8>, btree: &'a BTreeMap<Segment<8>, FlexIdRankList>) -> Self {
        Self {
            segment,
            parents: OnceCell::new(),
            children: OnceCell::new(),
            btree,
        }
    }

    ///このセグメントの親セグメントを持つFlexIdのRankを全て返す
    fn parents(&self) -> &FlexIdRankList {
        self.parents.get_or_init(|| {
            let mut result = FlexIdRankList::new();
            for parent_segment in self.segment.self_and_parents() {
                if let Some(v) = self.btree.get(&parent_segment) {
                    result |= v;
                }
            }
            result
        })
    }

    fn children(&self) -> &FlexIdRankList {
        self.children.get_or_init(|| {
            let mut result = FlexIdRankList::new();

            let end = self.segment.descendant_range_end();
            let range_scan = match end {
                Some(v) => self.btree.range(self.segment.clone()..v),
                None => self.btree.range(self.segment.clone()..),
            };

            for (_, flex_id_ranks) in range_scan {
                result |= flex_id_ranks;
            }
            result
        })
    }
}

///Setの中からあるFlexIDに関連するIDを効率的にスキャンする
#[derive(Debug)]
pub struct FlexIdScanPlan<'a> {
    ///各次元の情報
    f: Vec<SegmentNeighborhood<'a>>,
    x: Vec<SegmentNeighborhood<'a>>,
    y: Vec<SegmentNeighborhood<'a>>,
}

impl<'a> FlexIdScanPlan<'a> {
    ///新しくスキャナーを作成する
    /// VBitCoreからBTreeMapの参照をもらうため引数にcoreを取るが、構造体には保持しない
    pub fn new<T, S: SpatialId>(core: &'a VBitCore<T>, target: S) -> Self {
        let segmentation = target.segmentation();
        Self {
            f: segmentation
                .f
                .into_iter()
                .map(|segment| SegmentNeighborhood::new(segment, core.f()))
                .collect(),
            x: segmentation
                .x
                .into_iter()
                .map(|segment| SegmentNeighborhood::new(segment, core.x()))
                .collect(),
            y: segmentation
                .y
                .into_iter()
                .map(|segment| SegmentNeighborhood::new(segment, core.y()))
                .collect(),
        }
    }

    ///個別のスキャンを開始する
    pub fn scan(&self) -> impl Iterator<Item = FlexIdScanner<'_>> {
        self.f.iter().flat_map(move |f| {
            self.x.iter().flat_map(move |x| {
                self.y.iter().map(move |y| FlexIdScanner {
                    f,
                    x,
                    y,
                    parent: OnceCell::new(),
                    children: OnceCell::new(),
                })
            })
        })
    }
}

pub struct FlexIdScanner<'a> {
    ///各次元のデータを持っている
    f: &'a SegmentNeighborhood<'a>,
    x: &'a SegmentNeighborhood<'a>,
    y: &'a SegmentNeighborhood<'a>,

    //計算結果をキャッシュしておく
    parent: OnceCell<Option<FlexIdRank>>,
    children: OnceCell<FlexIdRankList>,
}

impl FlexIdScanner<'_> {
    pub fn flex_id(&self) -> FlexId {
        let f = self.as_f();
        let x = self.as_x();
        let y = self.as_y();
        FlexId::new_from_segments(f.clone(), x.clone(), y.clone())
    }

    ///親のFlexIdがあるかどうかを判定し、あればそのRankを返す
    /// Parentには自分と同じ形のFlexIdも含まれる
    /// ParentはSetの定義上、必ず0-1個である
    pub fn parent(&self) -> Option<FlexIdRank> {
        *self.parent.get_or_init(|| {
            let f = self.f.parents();
            let x = self.x.parents();
            let y = self.y.parents();
            let intersection = [f, x, y].intersection();

            intersection.iter().next()
        })
    }

    ///完全に自分含むFlexIdのRankを返す
    pub fn children(&self) -> &FlexIdRankList {
        self.children.get_or_init(|| {
            let f = self.f.children();
            let x = self.x.children();
            let y = self.y.children();
            [f, x, y].intersection()
        })
    }

    ///親でも子でもなく、部分的に重なっているFlexIdのRankを返す
    pub fn partial_overlaps(&self) -> FlexIdRankList {
        let mut all = self.all();
        all -= self.children();

        if let Some(parent_rank) = self.parent() {
            all.remove(parent_rank);
        }

        all
    }

    ///全ての重なりがあるFlexIDのRankを返す
    pub fn all(&self) -> FlexIdRankList {
        let f = self.f.parents() | self.f.children();
        let x = self.x.parents() | self.x.children();
        let y = self.y.parents() | self.y.children();
        [&f, &x, &y].intersection()
    }

    pub fn as_f(&self) -> &Segment<8> {
        &self.f.segment
    }
    pub fn as_x(&self) -> &Segment<8> {
        &self.x.segment
    }
    pub fn as_y(&self) -> &Segment<8> {
        &self.y.segment
    }
}
