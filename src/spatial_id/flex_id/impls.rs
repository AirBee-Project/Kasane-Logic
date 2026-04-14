use crate::{FlexId, RangeId, SingleId, SpatialId};

impl SpatialId for FlexId {
    fn f_min(&self) -> i32 {
        todo!()
    }

    fn f_max(&self) -> i32 {
        todo!()
    }

    fn xy_max(&self) -> u32 {
        todo!()
    }

    fn move_f(&mut self, by: i32) -> Result<(), crate::Error> {
        todo!()
    }

    fn move_x(&mut self, by: i32) {
        todo!()
    }

    fn move_y(&mut self, by: i32) -> Result<(), crate::Error> {
        todo!()
    }

    fn length_f_meters(&self) -> f64 {
        todo!()
    }

    fn length_x_meters(&self) -> f64 {
        todo!()
    }

    fn length_y_meters(&self) -> f64 {
        todo!()
    }

    fn spatial_center(&self) -> crate::Coordinate {
        todo!()
    }

    fn spatial_vertices(&self) -> [crate::Coordinate; 8] {
        todo!()
    }
}

impl From<FlexId> for RangeId {
    fn from(flex_id: FlexId) -> Self {
        RangeId::from(&flex_id)
    }
}

impl From<&FlexId> for RangeId {
    fn from(flex_id: &FlexId) -> Self {
        let max_z = flex_id
            .f_zoomlevel
            .max(flex_id.x_zoomlevel)
            .max(flex_id.y_zoomlevel);

        let scale_to_range = |val: i64, current_z: u8| -> [i64; 2] {
            let diff = max_z - current_z;
            let start = val << diff;
            let end = start + (1_i64 << diff) - 1;
            [start, end]
        };

        let f_range = scale_to_range(flex_id.f_index as i64, flex_id.f_zoomlevel);
        let x_range = scale_to_range(flex_id.x_index as i64, flex_id.x_zoomlevel);
        let y_range = scale_to_range(flex_id.y_index as i64, flex_id.y_zoomlevel);

        unsafe {
            RangeId::new_with_temporal_unchecked(
                max_z,
                [f_range[0] as i32, f_range[1] as i32],
                [x_range[0] as u32, x_range[1] as u32],
                [y_range[0] as u32, y_range[1] as u32],
                #[cfg(feature = "temporal")]
                flex_id.temporal().clone(),
            )
        }
    }
}

impl From<SingleId> for FlexId {
    fn from(value: SingleId) -> Self {
        FlexId::new_with_temporal(
            value.z(),
            value.f(),
            value.z(),
            value.x(),
            value.z(),
            value.y(),
            #[cfg(feature = "temporal")]
            value.temporal().clone(),
        )
        .unwrap()
    }
}

impl From<&SingleId> for FlexId {
    fn from(value: &SingleId) -> Self {
        FlexId::new_with_temporal(
            value.z(),
            value.f(),
            value.z(),
            value.x(),
            value.z(),
            value.y(),
            #[cfg(feature = "temporal")]
            value.temporal().clone(),
        )
        .unwrap()
    }
}
