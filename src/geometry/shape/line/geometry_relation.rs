use crate::{Coordinate, ExpandCoordinates, Line};

impl ExpandCoordinates for Line {
    fn expand_coordinates(&self) -> impl Iterator<Item = Coordinate> {
        self.points.into_iter()
    }
}
