use crate::Segment;
use crate::spatial_id::constants::MAX_ZOOM_LEVEL;
use crate::spatial_id::flex_id::segment::SegmentRelation;
pub mod impls;
pub mod segment;

/// # Warning!!!
/// この空間IDには[TemporalId]が実装されていません。他のIDから変換が行われた場合は時間に関する情報が失われます。注意してください。
///
///---
///
///FlexIdは拡張空間IDを表す。
///
/// 各インデックスを[Segment]を用いて表すことで、各次元で独立のズームレベルを持つことが可能です。
///
/// この型は `PartialOrd` / `Ord` を実装していますが、これは主に`BTreeSet` や `BTreeMap` などの順序付きコレクションでの格納・探索用です。実際の空間的な「大小」を意味するものではありません。
///
///
/// ```ignore
/// pub struct FlexId {
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
    // temporal_id: TemporalId,
}

impl FlexId {
    /// 新しく[FlexId]を作成する。
    pub fn new(
        f_zoomlevel: u8,
        f_index: i32,
        x_zoomlevel: u8,
        x_index: u32,
        y_zoomlevel: u8,
        y_index: u32,
    ) -> FlexId {
        Self::new_with_temporal(
            f_zoomlevel,
            f_index,
            x_zoomlevel,
            x_index,
            y_zoomlevel,
            y_index,
            // TemporalId::whole(),
        )
    }

    pub fn new_from_segments(f: Segment<8>, x: Segment<8>, y: Segment<8>) -> Self {
        FlexId {
            f,
            x,
            y,
            // temporal_id: TemporalId::whole(),
        }
    }

    pub fn new_with_temporal(
        f_zoomlevel: u8,
        f_index: i32,
        x_zoomlevel: u8,
        x_index: u32,
        y_zoomlevel: u8,
        y_index: u32,
        // temporal_id: TemporalId,
    ) -> FlexId {
        let f = Segment::from_f(f_zoomlevel, f_index);
        let x = Segment::from_xy(x_zoomlevel, x_index);
        let y = Segment::from_xy(y_zoomlevel, y_index);
        FlexId {
            f,
            x,
            y,
            // temporal_id,
        }
    }

    pub fn f(&self) -> (u8, i32) {
        self.f_segment().to_f()
    }

    pub fn x(&self) -> (u8, u32) {
        self.x_segment().to_xy()
    }

    pub fn y(&self) -> (u8, u32) {
        self.y_segment().to_xy()
    }

    /// Fインデックスのセグメントを参照する。
    pub fn f_segment(&self) -> &Segment<8> {
        &self.f
    }

    /// Xインデックスのセグメントを参照する。
    pub fn x_segment(&self) -> &Segment<8> {
        &self.x
    }

    /// Yインデックスのセグメントを参照する。
    pub fn y_segment(&self) -> &Segment<8> {
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

    ///[FlexId]同士の重なりがあるかないかを検証する
    pub fn is_intersects(&self, other: &FlexId) -> bool {
        self.f_segment().relation(other.f_segment()) != SegmentRelation::Disjoint
            && self.x_segment().relation(other.x_segment()) != SegmentRelation::Disjoint
            && self.y_segment().relation(other.y_segment()) != SegmentRelation::Disjoint
    }

    /// [FlexId]同士の重なり合っている部分を返す。
    pub fn intersection(&self, other: &FlexId) -> Option<FlexId> {
        let f = match self.f_segment().relation(other.f_segment()) {
            SegmentRelation::Equal => self.f_segment(),
            SegmentRelation::Ancestor => other.f_segment(),
            SegmentRelation::Descendant => self.f_segment(),
            SegmentRelation::Disjoint => {
                return None;
            }
        };

        let x = match self.x_segment().relation(other.x_segment()) {
            SegmentRelation::Equal => self.x_segment(),
            SegmentRelation::Ancestor => other.x_segment(),
            SegmentRelation::Descendant => self.x_segment(),
            SegmentRelation::Disjoint => {
                return None;
            }
        };

        let y = match self.y_segment().relation(other.y_segment()) {
            SegmentRelation::Equal => self.y_segment(),
            SegmentRelation::Ancestor => other.y_segment(),
            SegmentRelation::Descendant => self.y_segment(),
            SegmentRelation::Disjoint => {
                return None;
            }
        };

        Some(FlexId {
            f: f.clone(),
            x: x.clone(),
            y: y.clone(),
            // temporal_id: self.temporal_id.clone(),
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

        let f_diffs = self.f_segment().difference(&intersection.f);
        for f_seg in f_diffs {
            result.push(FlexId {
                f: f_seg,
                x: self.x.clone(),
                y: self.y.clone(),
                // temporal_id: self.temporal_id.clone(),
            });
        }

        let x_diffs = self.x_segment().difference(&intersection.x);
        for x_seg in x_diffs {
            result.push(FlexId {
                f: intersection.f.clone(),
                x: x_seg,
                y: self.y.clone(),
                // temporal_id: self.temporal_id.clone(),
            });
        }

        let y_diffs = self.y_segment().difference(&intersection.y);
        for y_seg in y_diffs {
            result.push(FlexId {
                f: intersection.f.clone(),
                x: intersection.x.clone(),
                y: y_seg,
                // temporal_id: self.temporal_id.clone(),
            });
        }

        result
    }

    ///[FlexId]が相手のFlexIdを含むかどうかを判定する。
    #[allow(dead_code)]
    fn contains(&self, other: &FlexId) -> bool {
        self.f_segment().relation(other.f_segment()) != SegmentRelation::Disjoint
            && self.f_segment().relation(other.f_segment()) != SegmentRelation::Descendant
            && self.x_segment().relation(other.x_segment()) != SegmentRelation::Disjoint
            && self.x_segment().relation(other.x_segment()) != SegmentRelation::Descendant
            && self.y_segment().relation(other.y_segment()) != SegmentRelation::Disjoint
            && self.y_segment().relation(other.y_segment()) != SegmentRelation::Descendant
    }
    ///Fセグメントが兄弟で、他が同じなFlexIdを返す
    pub fn f_sibling(&self) -> FlexId {
        FlexId {
            f: self.f.sibling(),
            x: self.x.clone(),
            y: self.y.clone(),
            // temporal_id: self.temporal_id.clone(),
        }
    }
    ///Xセグメントが兄弟で、他が同じなFlexIdを返す
    pub fn x_sibling(&self) -> FlexId {
        FlexId {
            f: self.f.clone(),
            x: self.x.sibling(),
            y: self.y.clone(),
            // temporal_id: self.temporal_id.clone(),
        }
    }
    ///Yセグメントが兄弟で、他が同じなFlexIdを返す
    pub fn y_sibling(&self) -> FlexId {
        FlexId {
            f: self.f.clone(),
            x: self.x.clone(),
            y: self.y.sibling(),
            // temporal_id: self.temporal_id.clone(),
        }
    }

    ///Fセグメントを親セグメントにした[FlexId]を返す。
    pub fn f_parent(&self) -> Option<FlexId> {
        Some(FlexId {
            f: self.f_segment().parent()?,
            x: self.x_segment().clone(),
            y: self.y_segment().clone(),
            // temporal_id: self.temporal_id.clone(),
        })
    }

    ///Xセグメントを親セグメントにした[FlexId]を返す。
    pub fn x_parent(&self) -> Option<FlexId> {
        Some(FlexId {
            f: self.f_segment().clone(),
            x: self.x_segment().parent()?,
            y: self.y_segment().clone(),
            // temporal_id: self.temporal_id.clone(),
        })
    }

    ///Yセグメントを親セグメントにした[FlexId]を返す。
    pub fn y_parent(&self) -> Option<FlexId> {
        Some(FlexId {
            f: self.f_segment().clone(),
            x: self.x_segment().clone(),
            y: self.y_segment().parent()?,
            // temporal_id: self.temporal_id.clone(),
        })
    }
}
