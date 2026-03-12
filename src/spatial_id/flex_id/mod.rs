use crate::spatial_id::constants::MAX_ZOOM_LEVEL;
use crate::spatial_id::flex_id::segment::SegmentRelation;
use crate::{RangeId, Segment};
pub mod impls;
pub mod segment;

///FlexIdは拡張空間IDを表す。
///
/// 各インデックスを[Segment]を用いて表すことで、各次元で独立のズームレベルを持つことが可能です。
///
/// この型は `PartialOrd` / `Ord` を実装していますが、これは主に`BTreeSet` や `BTreeMap` などの順序付きコレクションでの格納・探索用です。実際の空間的な「大小」を意味するものではありません。
///
///
/// ```ignore
/// pub struct RangeId {
///     f: Segment,
///     x: Segment,
///     y: Segment,
/// }
/// ```
#[derive(Clone, PartialEq, Debug, Eq, PartialOrd, Ord)]
pub struct FlexId {
    f: Segment<8>,
    x: Segment<8>,
    y: Segment<8>,
}

/// [FlexId]同士の関係を表します。
enum FlexIdRelation {
    /// 重なりがない。
    Disjoint,

    /// 重なりがある。
    Related,
}

impl FlexId {
    /// 新しく[FlexId]を作成する。
    pub fn new(f: Segment<8>, x: Segment<8>, y: Segment<8>) -> FlexId {
        FlexId { f, x, y }
    }

    /// [FlexId]を[RangeId]に変換する。
    pub fn range_id(&self) -> RangeId {
        self.clone().into()
    }

    /// Fインデックスのセグメントを参照する。
    pub fn f(&self) -> &Segment<8> {
        &self.f
    }

    /// Xインデックスのセグメントを参照する。
    pub fn x(&self) -> &Segment<8> {
        &self.x
    }

    /// Yインデックスのセグメントを参照する。
    pub fn y(&self) -> &Segment<8> {
        &self.y
    }

    /// このブロックの体積（最小単位ボクセルの数）を返す。
    pub fn volume(&self) -> u128 {
        let (fz, _) = self.f.to_f();
        let (xz, _) = self.x.to_xy();
        let (yz, _) = self.y.to_xy();

        let f_len = 1u128 << (MAX_ZOOM_LEVEL as u8 - fz);
        let x_len = 1u128 << (MAX_ZOOM_LEVEL as u8 - xz);
        let y_len = 1u128 << (MAX_ZOOM_LEVEL as u8 - yz);

        f_len * x_len * y_len
    }

    ///[FlexId]同士の関連を[FlexIdRelation]として返す。
    #[allow(dead_code)]
    fn relation(&self, other: &FlexId) -> FlexIdRelation {
        let f_relation = self.f().relation(other.f());
        let x_relation = self.x().relation(other.x());
        let y_relation = self.y().relation(other.y());

        if f_relation == SegmentRelation::Disjoint
            || x_relation == SegmentRelation::Disjoint
            || y_relation == SegmentRelation::Disjoint
        {
            FlexIdRelation::Disjoint
        } else {
            FlexIdRelation::Related
        }
    }

    /// [FlexId]同士の重なり合っている部分を返す。
    pub fn intersection(&self, other: &FlexId) -> Option<FlexId> {
        let f = match self.f().relation(other.f()) {
            SegmentRelation::Equal => self.f(),
            SegmentRelation::Ancestor => other.f(),
            SegmentRelation::Descendant => self.f(),
            SegmentRelation::Disjoint => {
                return None;
            }
        };

        let x = match self.x().relation(other.x()) {
            SegmentRelation::Equal => self.x(),
            SegmentRelation::Ancestor => other.x(),
            SegmentRelation::Descendant => self.x(),
            SegmentRelation::Disjoint => {
                return None;
            }
        };

        let y = match self.y().relation(other.y()) {
            SegmentRelation::Equal => self.y(),
            SegmentRelation::Ancestor => other.y(),
            SegmentRelation::Descendant => self.y(),
            SegmentRelation::Disjoint => {
                return None;
            }
        };

        Some(FlexId {
            f: f.clone(),
            x: x.clone(),
            y: y.clone(),
        })
    }

    /// [FlexId]から[FlexId]を引き、残った[FlexId]の集合を返す。
    pub fn difference(&self, other: &FlexId) -> Vec<FlexId> {
        let intersection = match self.intersection(other) {
            Some(i) => i,
            None => return vec![self.clone()],
        };

        if *self == intersection {
            return vec![];
        }

        let mut result = Vec::new();

        let f_diffs = self.f().difference(&intersection.f);
        for f_seg in f_diffs {
            result.push(FlexId {
                f: f_seg,
                x: self.x.clone(),
                y: self.y.clone(),
            });
        }

        let x_diffs = self.x().difference(&intersection.x);
        for x_seg in x_diffs {
            result.push(FlexId {
                f: intersection.f.clone(),
                x: x_seg,
                y: self.y.clone(),
            });
        }

        let y_diffs = self.y().difference(&intersection.y);
        for y_seg in y_diffs {
            result.push(FlexId {
                f: intersection.f.clone(),
                x: intersection.x.clone(),
                y: y_seg,
            });
        }

        result
    }

    ///[FlexId]が相手のFlexIdを含むかどうかを判定する。
    #[allow(dead_code)]
    fn contains(&self, other: &FlexId) -> bool {
        self.f().relation(other.f()) != SegmentRelation::Disjoint
            && self.f().relation(other.f()) != SegmentRelation::Descendant
            && self.x().relation(other.x()) != SegmentRelation::Disjoint
            && self.x().relation(other.x()) != SegmentRelation::Descendant
            && self.y().relation(other.y()) != SegmentRelation::Disjoint
            && self.y().relation(other.y()) != SegmentRelation::Descendant
    }
    ///Fセグメントが兄弟で、他が同じなFlexIdを返す
    pub fn f_sibling(&self) -> FlexId {
        FlexId {
            f: self.f.sibling(),
            x: self.x.clone(),
            y: self.y.clone(),
        }
    }
    ///Xセグメントが兄弟で、他が同じなFlexIdを返す
    pub fn x_sibling(&self) -> FlexId {
        FlexId {
            f: self.f.clone(),
            x: self.x.sibling(),
            y: self.y.clone(),
        }
    }
    ///Yセグメントが兄弟で、他が同じなFlexIdを返す
    pub fn y_sibling(&self) -> FlexId {
        FlexId {
            f: self.f.clone(),
            x: self.x.clone(),
            y: self.y.sibling(),
        }
    }

    ///Fセグメントを親セグメントにした[FlexId]を返す。
    pub fn f_parent(&self) -> Option<FlexId> {
        Some(FlexId::new(
            self.f().parent()?,
            self.x().clone(),
            self.y().clone(),
        ))
    }

    ///Xセグメントを親セグメントにした[FlexId]を返す。
    pub fn x_parent(&self) -> Option<FlexId> {
        Some(FlexId::new(
            self.f().clone(),
            self.x().parent()?,
            self.y().clone(),
        ))
    }

    ///Yセグメントを親セグメントにした[FlexId]を返す。
    pub fn y_parent(&self) -> Option<FlexId> {
        Some(FlexId::new(
            self.f().clone(),
            self.x().clone(),
            self.y().parent()?,
        ))
    }
}
