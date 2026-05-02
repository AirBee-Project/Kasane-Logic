use crate::{Coordinate, IterCoordinates, Line};

impl IterCoordinates for Line {
    fn iter_coordinates(&self) -> impl Iterator<Item = Coordinate> {
        self.points.into_iter()
    }
}
