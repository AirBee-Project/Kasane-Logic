use alloc::boxed::Box;

use crate::{FlexId, RangeId, SingleId, SpatialId, spatial_id::zoom_level::ZoomLevel};

impl From<FlexId> for RangeId {
    fn from(flex_id: FlexId) -> Self {
        RangeId::from(&flex_id)
    }
}

impl From<&FlexId> for RangeId {
    fn from(flex_id: &FlexId) -> Self {
        let max_z = flex_id
            .f_zoomlevel
            .get()
            .max(flex_id.x_zoomlevel.get())
            .max(flex_id.y_zoomlevel.get());

        let scale_to_range = |val: i64, current_z: u8| -> [i64; 2] {
            let diff = max_z - current_z;
            let start = val << diff;
            let end = start + (1_i64 << diff) - 1;
            [start, end]
        };

        let f_range = scale_to_range(flex_id.f_index as i64, flex_id.f_zoomlevel.get());
        let x_range = scale_to_range(flex_id.x_index as i64, flex_id.x_zoomlevel.get());
        let y_range = scale_to_range(flex_id.y_index as i64, flex_id.y_zoomlevel.get());

        #[cfg(feature = "temporal_id")]
        {
            RangeId::new_with_temporal(
                max_z,
                [f_range[0] as i32, f_range[1] as i32],
                [x_range[0] as u32, x_range[1] as u32],
                [y_range[0] as u32, y_range[1] as u32],
                flex_id.temporal().clone(),
            )
            .unwrap()
        }

        #[cfg(not(feature = "temporal_id"))]
        {
            RangeId::new(
                max_z,
                [f_range[0] as i32, f_range[1] as i32],
                [x_range[0] as u32, x_range[1] as u32],
                [y_range[0] as u32, y_range[1] as u32],
            )
            .unwrap()
        }
    }
}

impl From<SingleId> for FlexId {
    fn from(value: SingleId) -> Self {
        FlexId::from(&value)
    }
}

impl From<&SingleId> for FlexId {
    fn from(value: &SingleId) -> Self {
        FlexId {
            f_zoomlevel: ZoomLevel::new(value.z()).unwrap(),
            f_index: value.f(),
            x_zoomlevel: ZoomLevel::new(value.z()).unwrap(),
            x_index: value.x(),
            y_zoomlevel: ZoomLevel::new(value.z()).unwrap(),
            y_index: value.y(),
            temporal_id: value.temporal().clone(),
        }
    }
}

impl IntoIterator for FlexId {
    type Item = FlexId;
    type IntoIter = core::iter::Once<FlexId>;
    fn into_iter(self) -> Self::IntoIter {
        core::iter::once(self)
    }
}

impl FlexId {
    pub fn single_ids(self) -> Box<dyn Iterator<Item = SingleId>> {
        Box::new(RangeId::from(self).single_ids())
    }
}
