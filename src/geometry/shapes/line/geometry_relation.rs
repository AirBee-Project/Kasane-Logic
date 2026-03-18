use crate::{Coordinate, IntoCoordinates, Line};

impl IntoCoordinates for Line {
    fn into_coordinates(self) -> impl Iterator<Item = Coordinate> {
        self.points.into_iter()
    }

    fn iter_coordinates(&self) -> impl Iterator<Item = Coordinate> {
        self.points.into_iter()
    }
}
