use crate::{Coordinate, ExpandCoordinates, ExpandLines, Line, Triangle};

impl ExpandCoordinates for Triangle {
    fn expand_coordinates(&self) -> impl Iterator<Item = Coordinate> {
        self.points.into_iter()
    }
}

impl ExpandLines for Triangle {
    fn expand_lines(&self) -> impl Iterator<Item = Line> {
        [
            Line::new([self.points[0], self.points[1]]),
            Line::new([self.points[1], self.points[2]]),
            Line::new([self.points[2], self.points[0]]),
        ]
        .into_iter()
    }
}
