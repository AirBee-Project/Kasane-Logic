use crate::RangeId;
use crate::spatial_id::constants::MAX_ZOOM_LEVEL;
use crate::spatial_id::segment::SegmentRelation;
use crate::spatial_id::{FlexIds, segment::Segment};
use crate::spatial_id::{HyperRect, HyperRectSegments};

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
    f: Segment,
    x: Segment,
    y: Segment,
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
    pub fn new(f: Segment, x: Segment, y: Segment) -> FlexId {
        FlexId { f, x, y }
    }

    /// [FlexId]を[RangeId]に変換する。
    pub(crate) fn range_id(&self) -> RangeId {
        self.clone().into()
    }

    /// Fインデックスのセグメントを参照する。
    pub fn f(&self) -> &Segment {
        &self.f
    }

    /// Xインデックスのセグメントを参照する。
    pub fn x(&self) -> &Segment {
        &self.x
    }

    /// Yインデックスのセグメントを参照する。
    pub fn y(&self) -> &Segment {
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
    pub(crate) fn intersection(&self, other: &FlexId) -> Option<FlexId> {
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
    pub(crate) fn f_sibling(&self) -> FlexId {
        FlexId {
            f: self.f.sibling(),
            x: self.x.clone(),
            y: self.y.clone(),
        }
    }
    ///Xセグメントが兄弟で、他が同じなFlexIdを返す
    pub(crate) fn x_sibling(&self) -> FlexId {
        FlexId {
            f: self.f.clone(),
            x: self.x.sibling(),
            y: self.y.clone(),
        }
    }
    ///Yセグメントが兄弟で、他が同じなFlexIdを返す
    pub(crate) fn y_sibling(&self) -> FlexId {
        FlexId {
            f: self.f.clone(),
            x: self.x.clone(),
            y: self.y.sibling(),
        }
    }

    ///Fセグメントを親セグメントにした[FlexId]を返す。
    pub(crate) fn f_parent(&self) -> Option<FlexId> {
        Some(FlexId::new(
            self.f().parent()?,
            self.x().clone(),
            self.y().clone(),
        ))
    }

    ///Xセグメントを親セグメントにした[FlexId]を返す。
    pub(crate) fn x_parent(&self) -> Option<FlexId> {
        Some(FlexId::new(
            self.f().clone(),
            self.x().parent()?,
            self.y().clone(),
        ))
    }

    ///Yセグメントを親セグメントにした[FlexId]を返す。
    pub(crate) fn y_parent(&self) -> Option<FlexId> {
        Some(FlexId::new(
            self.f().clone(),
            self.x().clone(),
            self.y().parent()?,
        ))
    }
}

impl HyperRect for FlexId {
    fn segmentation(&self) -> HyperRectSegments {
        HyperRectSegments {
            f: vec![self.f.clone()],
            x: vec![self.x.clone()],
            y: vec![self.y.clone()],
        }
    }
}

impl From<FlexId> for RangeId {
    fn from(flex_id: FlexId) -> Self {
        let (f_z, f_dim) = flex_id.f.to_f();
        let (x_z, x_dim) = flex_id.x.to_xy();
        let (y_z, y_dim) = flex_id.y.to_xy();

        let max_z = f_z.max(x_z).max(y_z);

        let scale_to_range = |val: i64, current_z: u8| -> [i64; 2] {
            let diff = max_z - current_z;
            let start = val << diff;
            let end = start + (1_i64 << diff) - 1;
            [start, end]
        };

        let f_range = scale_to_range(f_dim as i64, f_z);
        let x_range = scale_to_range(x_dim as i64, x_z);
        let y_range = scale_to_range(y_dim as i64, y_z);

        unsafe {
            RangeId::new_unchecked(
                max_z,
                [f_range[0] as i32, f_range[1] as i32],
                [x_range[0] as u32, x_range[1] as u32],
                [y_range[0] as u32, y_range[1] as u32],
            )
        }
    }
}

impl From<FlexId>
    for (
        [u8; Segment::ARRAY_LENGTH],
        [u8; Segment::ARRAY_LENGTH],
        [u8; Segment::ARRAY_LENGTH],
    )
{
    ///
    fn from(value: FlexId) -> Self {
        (value.f.into(), value.x.into(), value.y.into())
    }
}

impl
    From<(
        [u8; Segment::ARRAY_LENGTH],
        [u8; Segment::ARRAY_LENGTH],
        [u8; Segment::ARRAY_LENGTH],
    )> for FlexId
{
    fn from(
        value: (
            [u8; Segment::ARRAY_LENGTH],
            [u8; Segment::ARRAY_LENGTH],
            [u8; Segment::ARRAY_LENGTH],
        ),
    ) -> Self {
        Self {
            f: value.0.into(),
            x: value.1.into(),
            y: value.2.into(),
        }
    }
}
