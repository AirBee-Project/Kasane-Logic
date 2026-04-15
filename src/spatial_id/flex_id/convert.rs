use crate::{
    FlexId, IntoFlexIds, IntoSingleIds, IterFlexIds, IterSingleIds, RangeId, SingleId, SpatialId,
};

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
                flex_id.temporal().clone(),
            )
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
        FlexId::new_with_temporal(
            value.z(),
            value.f(),
            value.z(),
            value.x(),
            value.z(),
            value.y(),
            value.temporal().clone(),
        )
        .unwrap()
    }
}

impl IntoFlexIds for FlexId {
    type IntoIter = std::iter::Once<FlexId>;
    fn into_flex_ids(self) -> Self::IntoIter {
        std::iter::once(self)
    }
}

impl IterFlexIds for FlexId {
    type Iter<'a> = std::iter::Once<FlexId>;
    fn iter_flex_ids(&self) -> Self::Iter<'_> {
        std::iter::once(self.clone())
    }
}

impl IntoSingleIds for FlexId {
    type IntoIter = Box<dyn Iterator<Item = SingleId>>;
    fn into_single_ids(self) -> Self::IntoIter {
        RangeId::from(self).into_single_ids()
    }
}

impl IterSingleIds for FlexId {
    type Iter<'a> = Box<dyn Iterator<Item = SingleId> + 'a>;
    fn iter_single_ids(&self) -> Self::Iter<'_> {
        RangeId::from(self).into_single_ids()
    }
}
