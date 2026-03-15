use crate::{FlexId, RangeId, Segment, SingleId, SpatialId, SpatialIds};

impl SpatialIds for FlexId {
    type SingleIdItem<'a> = SingleId;

    type RangeIdItem<'a> = RangeId;

    type FlexIdItem<'a> = &'a FlexId;

    fn single_ids(&self) -> impl Iterator<Item = Self::SingleIdItem<'_>> {
        self.range_id().single_ids()
    }

    fn range_ids(&self) -> impl Iterator<Item = Self::RangeIdItem<'_>> {}

    fn flex_ids(&self) -> impl Iterator<Item = Self::FlexIdItem<'_>> {
        std::iter::once(self)
    }

    fn optimize_single_ids(&self) -> impl Iterator<Item = Self::SingleIdItem<'_>> {
        todo!()
    }

    fn optimize_range_ids(&self) -> impl Iterator<Item = Self::RangeIdItem<'_>> {
        todo!()
    }

    fn optimize_flex_ids(&self) -> impl Iterator<Item = Self::FlexIdItem<'_>> {
        std::iter::once(self)
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

impl From<SingleId> for FlexId {
    fn from(value: SingleId) -> Self {
        let f = Segment::from_f(value.z(), value.f());
        let x = Segment::from_xy(value.z(), value.x());
        let y = Segment::from_xy(value.z(), value.y());
        FlexId::new(f, x, y)
    }
}

impl From<&SingleId> for FlexId {
    fn from(value: &SingleId) -> Self {
        let f = Segment::from_f(value.z(), value.f());
        let x = Segment::from_xy(value.z(), value.x());
        let y = Segment::from_xy(value.z(), value.y());
        FlexId::new(f, x, y)
    }
}
