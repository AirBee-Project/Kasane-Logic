use crate::RangeId;
use crate::spatial_id::segment::SegmentRelation;
use crate::spatial_id::{ToFlexId, segment::Segment};
#[derive(Clone, PartialEq, Debug)]
pub struct FlexId {
    f: Segment,
    x: Segment,
    y: Segment,
}

pub enum FlexIdRelation {
    Disjoint,
    Related,
}

impl FlexId {
    pub fn new(f: Segment, x: Segment, y: Segment) -> FlexId {
        FlexId { f, x, y }
    }

    pub fn decode(&self) -> RangeId {
        self.clone().into()
    }

    pub fn as_f(&self) -> &Segment {
        &self.f
    }

    pub fn as_x(&self) -> &Segment {
        &self.x
    }

    pub fn as_y(&self) -> &Segment {
        &self.y
    }

    ///FlexId同士の関連を返す関数
    pub fn relation(&self, other: &FlexId) -> FlexIdRelation {
        let f_relation = self.as_f().relation(other.as_f());
        let x_relation = self.as_x().relation(other.as_x());
        let y_relation = self.as_y().relation(other.as_y());

        if f_relation == SegmentRelation::Disjoint
            || x_relation == SegmentRelation::Disjoint
            || y_relation == SegmentRelation::Disjoint
        {
            FlexIdRelation::Disjoint
        } else {
            FlexIdRelation::Related
        }
    }

    pub fn intersection(&self, other: &FlexId) -> Option<FlexId> {
        let f = match self.as_f().relation(other.as_f()) {
            SegmentRelation::Equal => self.as_f(),
            SegmentRelation::Ancestor => other.as_f(),
            SegmentRelation::Descendant => self.as_f(),
            SegmentRelation::Disjoint => {
                return None;
            }
        };

        let x = match self.as_x().relation(other.as_x()) {
            SegmentRelation::Equal => self.as_x(),
            SegmentRelation::Ancestor => other.as_x(),
            SegmentRelation::Descendant => self.as_x(),
            SegmentRelation::Disjoint => {
                return None;
            }
        };

        let y = match self.as_y().relation(other.as_y()) {
            SegmentRelation::Equal => self.as_y(),
            SegmentRelation::Ancestor => other.as_y(),
            SegmentRelation::Descendant => self.as_y(),
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

    pub fn difference(&self, other: &FlexId) -> Vec<FlexId> {
        let intersection = match self.intersection(other) {
            Some(i) => i,
            None => return vec![self.clone()], // 排反ならAそのまま
        };

        if *self == intersection {
            return vec![];
        }

        let mut result = Vec::new();

        let f_diffs = self.as_x().difference(&intersection.x);
        for f_seg in f_diffs {
            result.push(FlexId {
                f: f_seg,
                x: self.x.clone(),
                y: self.y.clone(),
            });
        }

        let x_diffs = self.as_x().difference(&intersection.x);
        for x_seg in x_diffs {
            result.push(FlexId {
                f: intersection.f.clone(),
                x: x_seg,
                y: self.y.clone(),
            });
        }

        let y_diffs = self.as_y().difference(&intersection.y);
        for y_seg in y_diffs {
            result.push(FlexId {
                f: intersection.f.clone(),
                x: intersection.x.clone(),
                y: y_seg,
            });
        }

        result
    }

    pub fn contains(&self, other: &FlexId) -> bool {
        self.as_f().relation(other.as_f()) != SegmentRelation::Disjoint
            && self.as_f().relation(other.as_f()) != SegmentRelation::Descendant
            && self.as_x().relation(other.as_x()) != SegmentRelation::Disjoint
            && self.as_x().relation(other.as_x()) != SegmentRelation::Descendant
            && self.as_y().relation(other.as_y()) != SegmentRelation::Disjoint
            && self.as_y().relation(other.as_y()) != SegmentRelation::Descendant
    }

    ///f方向の親FlexIdを作成する
    pub fn f_parent(&self) -> Option<FlexId> {
        Some(FlexId::new(
            self.as_f().parent()?,
            self.as_x().clone(),
            self.as_y().clone(),
        ))
    }

    ///x方向の親FlexIdを作成する
    pub fn x_parent(&self) -> Option<FlexId> {
        Some(FlexId::new(
            self.as_f().clone(),
            self.as_x().parent()?,
            self.as_y().clone(),
        ))
    }

    ///y方向の親FlexIdを作成する
    pub fn y_parent(&self) -> Option<FlexId> {
        Some(FlexId::new(
            self.as_f().clone(),
            self.as_x().clone(),
            self.as_y().parent()?,
        ))
    }
}

impl ToFlexId for FlexId {
    fn to_flex_id(&self) -> impl Iterator<Item = FlexId> + '_ {
        std::iter::once(self.clone())
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
