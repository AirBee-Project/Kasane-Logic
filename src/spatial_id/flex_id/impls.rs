use crate::{FlexId, SpatialId, TemporalId};

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
        todo!()
    }

    fn temporal_mut(&mut self) -> &mut TemporalId {
        todo!()
    }
}
