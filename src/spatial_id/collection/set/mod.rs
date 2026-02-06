use std::collections::BTreeMap;

pub mod scanner;

use roaring::RoaringTreemap;

use crate::{
    FlexId, FlexIdRank, Segment,
    spatial_id::{FlexIds, collection::set::scanner::FlexIdScanPlan},
};

pub struct SetOnMemory {
    f: BTreeMap<Segment, RoaringTreemap>,
    x: BTreeMap<Segment, RoaringTreemap>,
    y: BTreeMap<Segment, RoaringTreemap>,
    main: BTreeMap<FlexIdRank, FlexId>,
}

impl SetOnMemory {
    pub fn insert<T: FlexIds>(&self, target: T) {
        let scanner = self.scanner(target);

        for flex_id_scanner in scanner.scan() {
            //もし、親に包まれていた場合はそのほかパターンを考える必要がない
            if flex_id_scanner.parent().is_some() {
                continue;
            }

            //
        }
    }

    ///Setの中からFlexIdを効率的にスキャンするようにする
    pub fn scanner<T: FlexIds>(&'_ self, target: T) -> FlexIdScanPlan<'_> {
        FlexIdScanPlan::new(self, target)
    }

    pub fn f(&self) -> &BTreeMap<Segment, RoaringTreemap> {
        &self.f
    }

    pub fn x(&self) -> &BTreeMap<Segment, RoaringTreemap> {
        &self.x
    }

    pub fn y(&self) -> &BTreeMap<Segment, RoaringTreemap> {
        &self.y
    }
}
