use alloc::boxed::Box;

use crate::{
    FlexId, IterFlexIds, IterSingleIds, RangeId, SingleId, SpatialId,
    spatial_id::zoom_level::ZoomLevel,
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
            RangeId::new(
                max_z,
                [f_range[0] as i32, f_range[1] as i32],
                [x_range[0] as u32, x_range[1] as u32],
                [y_range[0] as u32, y_range[1] as u32],
            )
            .map(|id| id.with_temporal(flex_id.temporal().clone()))
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



impl IterFlexIds for FlexId {
    type Iter<'a> = core::iter::Once<FlexId>;
    fn iter_flex_ids(&self) -> Self::Iter<'_> {
        core::iter::once(self.clone())
    }
}



impl IterSingleIds for FlexId {
    type Iter<'a> = Box<dyn Iterator<Item = SingleId> + 'a>;
    fn iter_single_ids(&self) -> Self::Iter<'_> {
        let range = RangeId::from(self);
        let z = range.z();
        let f_range = range.f()[0]..=range.f()[1];
        let y_range = range.y()[0]..=range.y()[1];
        let x_0 = range.x()[0];
        let x_1 = range.x()[1];
        let t_id = range.temporal().clone();

        let iter = f_range.flat_map(move |f| {
            let y_range = y_range.clone();
            let t_id_inner = t_id.clone();
            let x_iter = if x_0 <= x_1 {
                (x_0..=x_1).collect::<alloc::vec::Vec<_>>()
            } else {
                (x_0..=crate::spatial_id::zoom_level::ZoomLevel::new(z).unwrap().xy_max())
                    .chain(0..=x_1)
                    .collect::<alloc::vec::Vec<_>>()
            };

            x_iter.into_iter().flat_map(move |x| {
                let t_id = t_id_inner.clone();
                y_range.clone().map(move |y: u32| {
                    #[cfg(feature = "temporal_id")]
                    {
                        SingleId::new(z, f, x, y).unwrap().with_temporal(t_id.clone())
                    }
                    #[cfg(not(feature = "temporal_id"))]
                    {
                        SingleId::new(z, f, x, y).unwrap()
                    }
                })
            })
        });
        Box::new(iter)
    }
}
