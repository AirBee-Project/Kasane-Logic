use crate::spatial_id::{
    range::RangeId,
    segment::{
        Segment,
        encode::{EncodeSegment, SegmentRelation},
    },
};

pub struct EncodeId {
    pub(crate) f: EncodeSegment,
    pub(crate) x: EncodeSegment,
    pub(crate) y: EncodeSegment,
}

pub enum EncodeIdRelation {
    Disjoint,
    Intersecting,
}

impl EncodeId {
    ///[RangeId]に戻す
    pub fn decode(&self) -> RangeId {
        let f_seg = Segment::<i32>::from(self.f.clone());
        let x_seg = Segment::<u32>::from(self.x.clone());
        let y_seg = Segment::<u32>::from(self.y.clone());

        let max_z = f_seg.as_z().max(x_seg.as_z().max(y_seg.as_z()));

        let scale_to_range = |val: i64, current_z: u8, target_z: u8| -> [i64; 2] {
            let diff = target_z - current_z;
            let scale = 1_i64 << diff;

            let start = val * scale;
            let end = start + scale - 1;

            [start, end]
        };

        let f_range = scale_to_range(f_seg.as_dimension() as i64, f_seg.as_z(), max_z);
        let x_range = scale_to_range(x_seg.as_dimension() as i64, x_seg.as_z(), max_z);
        let y_range = scale_to_range(y_seg.as_dimension() as i64, y_seg.as_z(), max_z);

        RangeId {
            z: max_z,
            f: [f_range[0] as i32, f_range[1] as i32],
            x: [x_range[0] as u32, x_range[1] as u32],
            y: [y_range[0] as u32, y_range[1] as u32],
        }
    }

    pub fn as_f(&self) -> &EncodeSegment {
        &self.f
    }

    pub fn as_x(&self) -> &EncodeSegment {
        &self.x
    }

    pub fn as_y(&self) -> &EncodeSegment {
        &self.y
    }

    ///EncodeId同士の関連を返す関数
    pub fn relation(&self, other: &EncodeId) -> EncodeIdRelation {
        let f_relation = self.as_f().relation(other.as_f());
        let x_relation = self.as_x().relation(other.as_x());
        let y_relation = self.as_y().relation(other.as_y());

        if f_relation == SegmentRelation::Disjoint
            || x_relation == SegmentRelation::Disjoint
            || y_relation == SegmentRelation::Disjoint
        {
            EncodeIdRelation::Disjoint
        } else {
            EncodeIdRelation::Intersecting
        }
    }

    pub fn intersection(&self, other: &EncodeId) -> Option<EncodeId> {
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

        Some(EncodeId {
            f: f.clone(),
            x: x.clone(),
            y: y.clone(),
        })
    }

    pub fn difference(&self, other: &EncodeId) -> Option<EncodeId> {
        todo!()
    }

    pub fn merge() -> impl Iterator<Item = EncodeId> {
        std::iter::empty()
    }
}
