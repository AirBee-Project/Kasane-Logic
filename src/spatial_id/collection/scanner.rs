use std::{
    cell::OnceCell,
    collections::{BTreeMap, HashMap},
};

use roaring::RoaringTreemap;

use crate::{
    FlexId,
    FlexIdRank,
    Segment,
    spatial_id::{FlexIds, helpers::fast_intersect}, // SetOnMemoryの依存は削除
};

///このTraitを実装していると、効率的に関連のある[FlexId]をスキャンする関数などを提供する
pub trait Scanner<'a>: Sized {
    fn f(&self) -> &'a BTreeMap<Segment, RoaringTreemap>;
    fn x(&self) -> &'a BTreeMap<Segment, RoaringTreemap>;
    fn y(&self) -> &'a BTreeMap<Segment, RoaringTreemap>;
    fn main(&self) -> &'a HashMap<FlexIdRank, FlexId>;

    ///[FlexIdScanPlan]型を返す
    fn flex_id_scan_plan<T: FlexIds>(&'a self, target: T) -> FlexIdScanPlan<'a, Self> {
        FlexIdScanPlan::new(self, target)
    }
}

#[derive(Debug)]
///関連あるセグメントを調べ、キャッシュしておく型
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

#[derive(Debug)]
pub struct FlexIdScanPlan<'a, S: Scanner<'a>> {
    scanner: &'a S,
    f: Vec<SegmentFamily<'a>>,
    x: Vec<SegmentFamily<'a>>,
    y: Vec<SegmentFamily<'a>>,
}

impl<'a, S> FlexIdScanPlan<'a, S>
where
    S: Scanner<'a>,
{
    pub fn new<T: FlexIds>(scanner: &'a S, target: T) -> Self {
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
            scanner: scanner,
        }
    }

    pub fn scan<'b>(&'b self) -> impl Iterator<Item = FlexIdScanner<'b, 'a, S>> {
        self.f.iter().flat_map(move |f| {
            self.x.iter().flat_map(move |x| {
                self.y.iter().map(move |y| FlexIdScanner {
                    set: self.scanner,
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
pub struct FlexIdScanner<'b, 'a, S: Scanner<'a>> {
    set: &'a S,

    // ここが重要: 参照自体は 'b、中身のデータは 'a
    f: &'b SegmentFamily<'a>,
    x: &'b SegmentFamily<'a>,
    y: &'b SegmentFamily<'a>,

    parent: OnceCell<Option<FlexIdRank>>,
    children: OnceCell<RoaringTreemap>,
}

impl<'b, 'a, S: Scanner<'a>> FlexIdScanner<'b, 'a, S> {
    ///対象となる[FlexId]
    pub fn flex_id(&self) -> FlexId {
        let f = self.as_f();
        let x = self.as_x();
        let y = self.as_y();
        FlexId::new(f.clone(), x.clone(), y.clone())
    }

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

    pub fn children(&self) -> &RoaringTreemap {
        self.children.get_or_init(|| {
            let f = self.f.children();
            let x = self.x.children();
            let y = self.y.children();
            fast_intersect([f, x, y])
        })
    }

    pub fn partial_overlaps(&self) -> RoaringTreemap {
        let mut all = self.all();
        all -= self.children();
        if let Some(parent_rank) = self.parent() {
            all.remove(parent_rank as u64);
        }

        all
    }

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
