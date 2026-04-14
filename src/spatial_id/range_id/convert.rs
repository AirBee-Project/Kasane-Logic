use crate::{
    FlexId, IntoFlexIds, IntoSingleIds, IterFlexIds, IterSingleIds, RangeId, Segment, SingleId,
    SpatialId, XY_MAX,
};

impl From<SingleId> for RangeId {
    fn from(id: SingleId) -> Self {
        RangeId::from(&id)
    }
}

impl From<&SingleId> for RangeId {
    fn from(id: &SingleId) -> Self {
        RangeId {
            z: id.z(),
            f: [id.f(), id.f()],
            x: [id.x(), id.x()],
            y: [id.y(), id.y()],

            temporal_id: id.temporal().clone(),
        }
    }
}

impl IntoSingleIds for RangeId {
    type IntoIter = Box<dyn Iterator<Item = SingleId>>;
    fn into_single_ids(self) -> Self::IntoIter {
        let z = self.z;
        let f_range = self.f[0]..=self.f[1];
        let y_range = self.y[0]..=self.y[1];
        let t_id = self.temporal_id.clone();

        let iter = f_range.flat_map(move |f| {
            let y_range = y_range.clone();
            let t_id = t_id.clone();

            let x_iter = if self.x[0] <= self.x[1] {
                (self.x[0]..=self.x[1]).collect::<Vec<_>>()
            } else {
                (self.x[0]..=XY_MAX[z as usize])
                    .chain(0..=self.x[1])
                    .collect::<Vec<_>>()
            };

            x_iter.into_iter().flat_map(move |x| {
                let t_id = t_id.clone();
                y_range.clone().map(move |y: u32| unsafe {
                    SingleId::new_with_temporal_unchecked(z, f, x, y, t_id.clone())
                })
            })
        });
        Box::new(iter)
    }
}

impl IterSingleIds for RangeId {
    type Iter<'a> = Box<dyn Iterator<Item = SingleId> + 'a>;
    fn iter_single_ids(&self) -> Self::Iter<'_> {
        let z = self.z;
        let f_range = self.f[0]..=self.f[1];
        let y_range = self.y[0]..=self.y[1];

        let iter = f_range.flat_map(move |f| {
            let y_range = y_range.clone();

            let x_iter = if self.x[0] <= self.x[1] {
                (self.x[0]..=self.x[1]).collect::<Vec<_>>()
            } else {
                (self.x[0]..=XY_MAX[z as usize])
                    .chain(0..=self.x[1])
                    .collect::<Vec<_>>()
            };

            x_iter.into_iter().flat_map(move |x| {
                y_range.clone().map(move |y: u32| unsafe {
                    SingleId::new_with_temporal_unchecked(z, f, x, y, self.temporal_id.clone())
                })
            })
        });
        Box::new(iter)
    }
}

impl IntoFlexIds for RangeId {
    type IntoIter = Box<dyn Iterator<Item = FlexId>>;

    fn into_flex_ids(self) -> Self::IntoIter {
        let f_list: Vec<_> = Segment::<8>::split_f(self.z, self.f).collect();

        let x_list: Vec<_> = if self.x[0] <= self.x[1] {
            Segment::<8>::split_xy(self.z, self.x).collect()
        } else {
            Segment::<8>::split_xy(self.z, [self.x[0], XY_MAX[self.z as usize]])
                .chain(Segment::<8>::split_xy(self.z, [0, self.x[1]]))
                .collect()
        };
        let y_list: Vec<_> = Segment::<8>::split_xy(self.z, self.y).collect();

        let iter = f_list.into_iter().flat_map(move |(f_z, f_i)| {
            let y_list_inner = y_list.clone();
            x_list.clone().into_iter().flat_map(move |(x_z, x_i)| {
                let y_list_inner = y_list_inner.clone();
                y_list_inner
                    .into_iter()
                    .map(move |(y_z, y_i)| FlexId::new(f_z, f_i, x_z, x_i, y_z, y_i).unwrap())
            })
        });
        Box::new(iter)
    }
}

impl IterFlexIds for RangeId {
    type Iter<'a> = Box<dyn Iterator<Item = FlexId> + 'a>;

    fn iter_flex_ids(&self) -> Self::Iter<'_> {
        self.clone().into_flex_ids()
    }
}
