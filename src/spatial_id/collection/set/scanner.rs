use std::{
    cell::{OnceCell, RefCell},
    collections::BTreeMap,
};

use roaring::RoaringTreemap;

use crate::{
    FlexIdRank, Segment,
    spatial_id::{FlexIds, collection::set::SetOnMemory},
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
}

impl FlexIdScanner<'_> {
    ///親のFlexIdがあるかどうかを判定し、あればそのRankを返す
    pub fn parent(&self) -> Option<FlexIdRank> {
        let f = self.f.parents();
        let x = self.x.parents();
        let y = self.y.parents();
        let intersection = f & x & y;
        intersection.iter().next()
    }
}
