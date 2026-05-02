use crate::{Coordinate, IntoCoordinates, IntoLines, Line, Triangle};

impl IntoCoordinates for Triangle {
    fn into_coordinates(self) -> impl Iterator<Item = Coordinate> {
        self.points.into_iter()
    }

    fn iter_coordinates(&self) -> impl Iterator<Item = Coordinate> {
        self.points.into_iter()
    }
}

impl IntoLines for Triangle {
    fn into_lines(self) -> impl Iterator<Item = Line> {
        [
            Line::new([self.points[0], self.points[1]]),
            Line::new([self.points[1], self.points[2]]),
            Line::new([self.points[2], self.points[0]]),
        ]
        .into_iter()
    }

    fn iter_lines(&self) -> impl Iterator<Item = Line> {
        [
            Line::new([self.points[0], self.points[1]]),
            Line::new([self.points[1], self.points[2]]),
            Line::new([self.points[2], self.points[0]]),
        ]
        .into_iter()
    }
}
