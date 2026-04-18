use std::fmt;

use crate::{FlexId, SpatialId, TemporalId};

impl fmt::Display for FlexId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //空間の情報の書き込み
        write!(
            f,
            "{}/{}|{}/{}|{}/{}",
            self.f_zoomlevel,
            self.f_index,
            self.x_zoomlevel,
            self.x_index,
            self.y_zoomlevel,
            self.y_index
        )?;

        //時間の情報があれば書き込み
        if !self.temporal_id.is_whole() {
            write!(f, "_{}", self.temporal_id)?;
        };
        Ok(())
    }
}

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

    fn move_f(&mut self, _by: i32) -> Result<(), crate::Error> {
        todo!()
    }

    fn move_x(&mut self, _by: i32) {
        todo!()
    }

    fn move_y(&mut self, _by: i32) -> Result<(), crate::Error> {
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

    fn temporal(&self) -> &TemporalId {
        &self.temporal_id
    }

    fn temporal_mut(&mut self) -> &mut TemporalId {
        &mut self.temporal_id
    }
}
