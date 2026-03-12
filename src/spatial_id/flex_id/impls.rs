use crate::{Block, FlexId, RangeId, spatial_id::BlockSegments};

impl Block for FlexId {
    fn segmentation(&self) -> BlockSegments {
        BlockSegments {
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

impl From<FlexId> for ([u8; 8], [u8; 8], [u8; 8]) {
    fn from(value: FlexId) -> Self {
        (value.f.into(), value.x.into(), value.y.into())
    }
}

impl From<([u8; 8], [u8; 8], [u8; 8])> for FlexId {
    fn from(value: ([u8; 8], [u8; 8], [u8; 8])) -> Self {
        Self {
            f: value.0.into(),
            x: value.1.into(),
            y: value.2.into(),
        }
    }
}
