use crate::{
    Coordinate, Ecef, Error, F_MAX, F_MIN, FlexId, RangeId, SingleId, SpatialId, SpatialIds,
    TemporalId, XY_MAX, spatial_id::helpers,
};

impl From<FlexId> for RangeId {
    fn from(flex_id: FlexId) -> Self {
        RangeId::from(&flex_id)
    }
}

impl From<&FlexId> for RangeId {
    fn from(flex_id: &FlexId) -> Self {
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
            temporal_id: TemporalId::whole(),
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
            value.temporal().clone(),
        )
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
    }
}

impl SpatialId for FlexId {
    fn f_min(&self) -> i32 {
        let (z, _) = self.f_segment().to_f();
        F_MIN[z as usize]
    }

    fn f_max(&self) -> i32 {
        let (z, _) = self.f_segment().to_f();
        F_MAX[z as usize]
    }

    fn xy_max(&self) -> u32 {
        let (x_z, _) = self.x_segment().to_xy();
        let (y_z, _) = self.y_segment().to_xy();
        XY_MAX[x_z as usize].max(XY_MAX[y_z as usize])
    }

    fn move_f(&mut self, by: i32) -> Result<(), Error> {
        let (z, f) = self.f_segment().to_f();
        let new_f = f.checked_add(by).ok_or(Error::FOutOfRange {
            f: if by >= 0 { i32::MAX } else { i32::MIN },
            z,
        })?;

        if new_f < self.f_min() || new_f > self.f_max() {
            return Err(Error::FOutOfRange { f: new_f, z });
        }

        self.f = crate::Segment::from_f(z, new_f);
        Ok(())
    }

    fn move_x(&mut self, by: i32) {
        let (z, x) = self.x_segment().to_xy();
        let wrapped = (x as i32 + by).rem_euclid(XY_MAX[z as usize] as i32);
        self.x = crate::Segment::from_xy(z, wrapped as u32);
    }

    fn move_y(&mut self, by: i32) -> Result<(), Error> {
        let (z, y) = self.y_segment().to_xy();
        let new_y = if by >= 0 {
            y.checked_add(by as u32)
                .ok_or(Error::YOutOfRange { y: u32::MAX, z })?
        } else {
            y.checked_sub((-by) as u32)
                .ok_or(Error::YOutOfRange { y: 0, z })?
        };

        if new_y > XY_MAX[z as usize] {
            return Err(Error::YOutOfRange { y: new_y, z });
        }

        self.y = crate::Segment::from_xy(z, new_y);
        Ok(())
    }

    fn length_f_meters(&self) -> f64 {
        let (z, _) = self.f_segment().to_f();
        2_i32.pow(25 - z as u32) as f64
    }

    fn length_x_meters(&self) -> f64 {
        let (z, _) = self.x_segment().to_xy();
        let ecef: Ecef = self.spatial_center().into();
        let r = (ecef.x() * ecef.x() + ecef.y() * ecef.y()).sqrt();
        r * 2.0 * std::f64::consts::PI / (2_i32.pow(z as u32) as f64)
    }

    fn length_y_meters(&self) -> f64 {
        let (z, _) = self.y_segment().to_xy();
        let ecef: Ecef = self.spatial_center().into();
        let r = (ecef.x() * ecef.x() + ecef.y() * ecef.y()).sqrt();
        r * 2.0 * std::f64::consts::PI / (2_i32.pow(z as u32) as f64)
    }

    fn spatial_center(&self) -> Coordinate {
        let (f_z, f_dim) = self.f_segment().to_f();
        let (x_z, x_dim) = self.x_segment().to_xy();
        let (y_z, y_dim) = self.y_segment().to_xy();

        unsafe {
            Coordinate::new_unchecked(
                helpers::latitude(y_dim as f64 + 0.5, y_z),
                helpers::longitude(x_dim as f64 + 0.5, x_z),
                helpers::altitude(f_dim as f64 + 0.5, f_z),
            )
        }
    }

    fn spatial_vertices(&self) -> [Coordinate; 8] {
        let (f_z, f_dim) = self.f_segment().to_f();
        let (x_z, x_dim) = self.x_segment().to_xy();
        let (y_z, y_dim) = self.y_segment().to_xy();

        let longitudes = [
            helpers::longitude(x_dim as f64, x_z),
            helpers::longitude(x_dim as f64 + 1.0, x_z),
        ];
        let latitudes = [
            helpers::latitude(y_dim as f64, y_z),
            helpers::latitude(y_dim as f64 + 1.0, y_z),
        ];
        let altitudes = [
            helpers::altitude(f_dim as f64, f_z),
            helpers::altitude(f_dim as f64 + 1.0, f_z),
        ];

        let mut out = [Coordinate::default(); 8];
        let mut i = 0;
        for f_i in 0..2 {
            for y_i in 0..2 {
                for x_i in 0..2 {
                    out[i]
                        .set_longitude(longitudes[x_i])
                        .expect("longitude must be within valid range");
                    out[i]
                        .set_latitude(latitudes[y_i])
                        .expect("latitude must be within valid range");
                    out[i]
                        .set_altitude(altitudes[f_i])
                        .expect("altitude must be within valid range");
                    i += 1;
                }
            }
        }

        out
    }

    fn temporal(&self) -> &TemporalId {
        &self.temporal_id
    }

    fn temporal_mut(&mut self) -> &mut TemporalId {
        &mut self.temporal_id
    }
}

impl SpatialIds for FlexId {
    type SingleItem<'a> = SingleId;

    type RangeItem<'a> = RangeId;

    type FlexItem<'a> = &'a FlexId;

    fn single_ids(&self) -> impl Iterator<Item = Self::SingleItem<'_>> {
        //Todo:最小個数になるように改良
        let range_id = RangeId::from(self);
        let items: Vec<_> = range_id.single_ids().collect();
        items.into_iter()
    }

    fn range_ids(&self) -> impl Iterator<Item = Self::RangeItem<'_>> {
        std::iter::once(RangeId::from(self))
    }

    fn flex_ids(&self) -> impl Iterator<Item = Self::FlexItem<'_>> {
        std::iter::once(self)
    }
}
