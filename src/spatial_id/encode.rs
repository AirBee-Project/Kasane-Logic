use crate::spatial_id::{
    range::RangeId,
    segment::{Segment, encode::EncodeSegment},
};

pub struct EncodeId {
    pub(crate) f: EncodeSegment,
    pub(crate) x: EncodeSegment,
    pub(crate) y: EncodeSegment,
}

impl EncodeId {
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
}
