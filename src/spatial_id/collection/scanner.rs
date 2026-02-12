use std::{cell::OnceCell, collections::BTreeMap};

use roaring::RoaringTreemap;

use crate::{
    FlexId, FlexIdRank, Segment,
    spatial_id::{FlexIds, helpers::fast_intersect},
};

///このTraitを実装していると、効率的に関連のある[FlexId]をスキャンする関数などを提供する
pub trait Scanner: Sized {
    fn f(&self) -> &BTreeMap<Segment, RoaringTreemap>;
    fn x(&self) -> &BTreeMap<Segment, RoaringTreemap>;
    fn y(&self) -> &BTreeMap<Segment, RoaringTreemap>;

    ///[FlexIdScanPlan]型を返す
    fn flex_id_scan_plan<T: FlexIds>(&'_ self, target: T) -> FlexIdScanPlan<'_> {
        FlexIdScanPlan::new(self, target)
    }

    ///Targetと全く同じ形のFlexIdを見つけ、そのFlexIdRankを返す
    fn find(&self, target: FlexId) -> Option<FlexIdRank> {
        let f = self.f().get(target.as_f())?;
        let x = self.x().get(target.as_x())?;
        let y = self.y().get(target.as_y())?;
        fast_intersect([f, x, y]).iter().next().clone()
    }
}

#[derive(Debug)]
///各次元の親と子をキャッシュしておく役割を持つ
pub struct FlexIdScanPlan<'a> {
    f: Vec<SegmentFamily<'a>>,
    x: Vec<SegmentFamily<'a>>,
    y: Vec<SegmentFamily<'a>>,
}

impl<'a> FlexIdScanPlan<'a> {
    pub fn new<T: FlexIds, S: Scanner>(scanner: &'a S, target: T) -> Self {
        let segmentation = target.segmentation();
        Self {
            f: segmentation
                .f
                .into_iter()
                .map(|segment| SegmentFamily::new(segment, scanner.f()))
                .collect(),
            x: segmentation
                .x
                .into_iter()
                .map(|segment| SegmentFamily::new(segment, scanner.x()))
                .collect(),
            y: segmentation
                .y
                .into_iter()
                .map(|segment| SegmentFamily::new(segment, scanner.y()))
                .collect(),
        }
    }

    pub fn scan<'b>(&'b self) -> impl Iterator<Item = FlexIdScanner<'b, 'a>> {
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

///ある[FlexId]について効率的にスキャンを行う型
pub struct FlexIdScanner<'b, 'a> {
    f: &'b SegmentFamily<'a>,
    x: &'b SegmentFamily<'a>,
    y: &'b SegmentFamily<'a>,

    parent: OnceCell<Option<FlexIdRank>>,
    children: OnceCell<RoaringTreemap>,
}

impl<'b, 'a> FlexIdScanner<'b, 'a> {
    ///対象となる[FlexId]
    pub fn flex_id(&self) -> FlexId {
        let f = self.f();
        let x = self.x();
        let y = self.y();
        FlexId::new(f.clone(), x.clone(), y.clone())
    }

    ///親の[FlexId]を取得する
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
            intersection.iter().next().map(|id| id as FlexIdRank)
        })
    }

    ///子の[FlexId]を取得する
    pub fn children(&self) -> &RoaringTreemap {
        self.children.get_or_init(|| {
            let f = self.f.children();
            let x = self.x.children();
            let y = self.y.children();
            fast_intersect([f, x, y])
        })
    }

    ///親でも子でもないが、重なりがある[FlexId]を取得する
    pub fn partial_overlaps(&self) -> RoaringTreemap {
        let mut all = self.all();
        all -= self.children();
        if let Some(parent_rank) = self.parent() {
            all.remove(parent_rank as u64);
        }

        all
    }

    ///重なりがある全ての[FlexId]を取得する
    pub fn all(&self) -> RoaringTreemap {
        let f = self.f.parents() | self.f.children();
        let x = self.x.parents() | self.x.children();
        let y = self.y.parents() | self.y.children();
        fast_intersect([&f, &x, &y])
    }

    pub fn f(&self) -> &Segment {
        &self.f.segment
    }
    pub fn x(&self) -> &Segment {
        &self.x.segment
    }
    pub fn y(&self) -> &Segment {
        &self.y.segment
    }
}

#[derive(Debug)]
///ある[FlexId]の関連あるセグメントを調べ、キャッシュしておく型
pub struct SegmentFamily<'a> {
    segment: Segment,
    parents: OnceCell<RoaringTreemap>,
    children: OnceCell<RoaringTreemap>,
    btree: &'a BTreeMap<Segment, RoaringTreemap>,
}

impl<'a> SegmentFamily<'a> {
    ///[SegmentFamily]を作成する
    fn new(segment: Segment, btree: &'a BTreeMap<Segment, RoaringTreemap>) -> Self {
        Self {
            segment,
            parents: OnceCell::new(),
            children: OnceCell::new(),
            btree,
        }
    }

    ///親を探す
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

    ///子を探す
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
