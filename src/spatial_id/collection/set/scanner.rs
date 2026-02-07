use std::{cell::OnceCell, collections::BTreeMap};

use roaring::RoaringTreemap;

use crate::{
    FlexId, FlexIdRank, Segment,
    spatial_id::{FlexIds, collection::set::SetOnMemory, helpers::fast_intersect},
};

#[derive(Debug)]
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
                if let Some(v) = self.btree.get(&parent_segment) {
                    result |= v;
                }
            }
            result
        })
    }

    fn children(&self) -> &RoaringTreemap {
        self.children.get_or_init(|| {
            let mut result = RoaringTreemap::new();

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
                    parent: OnceCell::new(),
                    children: OnceCell::new(),
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
    pub fn flex_id(&self) -> FlexId {
        let f = self.as_f();
        let x = self.as_x();
        let y = self.as_y();

        FlexId::new(f.clone(), x.clone(), y.clone())
    }

    ///親のFlexIdがあるかどうかを判定し、あればそのRankを返す
    /// Parentには自分と同じ形のFlexIdも含まれる
    /// ParentはSetの定義上、必ず0-1個である
    pub fn parent(&self) -> Option<FlexIdRank> {
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
    pub fn children(&self) -> &RoaringTreemap {
        self.children.get_or_init(|| {
            let f = self.f.children();
            let x = self.x.children();
            let y = self.y.children();
            fast_intersect([f, x, y])
        })
    }

    ///親でも子でもなく、部分的に重なっているFlexIdのRankを返す
    pub fn partial_overlaps(&self) -> RoaringTreemap {
        let mut all = self.all();
        all -= self.children();

        if let Some(parent_rank) = self.parent() {
            all.remove(parent_rank);
        }

        all
    }

    ///全ての重なりがあるFlexIDのRankを返す
    pub fn all(&self) -> RoaringTreemap {
        let f = self.f.parents() | self.f.children();
        let x = self.x.parents() | self.x.children();
        let y = self.y.parents() | self.y.children();
        fast_intersect([&f, &x, &y])
    }

    pub fn as_f(&self) -> &Segment {
        &self.f.segment
    }
    pub fn as_x(&self) -> &Segment {
        &self.x.segment
    }
    pub fn as_y(&self) -> &Segment {
        &self.y.segment
    }
}
