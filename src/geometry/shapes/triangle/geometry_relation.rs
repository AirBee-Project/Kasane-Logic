use crate::{Coordinate, Line, Triangle};

//[Coordinate]への変換
impl Into<Box<dyn Iterator<Item = Coordinate>>> for Triangle {
    fn into(self) -> Box<dyn Iterator<Item = Coordinate>> {
        let iter = self.points.into_iter();
        Box::new(iter)
    }
}

impl Into<Box<dyn Iterator<Item = Coordinate>>> for &Triangle {
    fn into(self) -> Box<dyn Iterator<Item = Coordinate>> {
        let iter = self.points.into_iter();
        Box::new(iter)
    }
}

impl<'a> Into<Box<dyn Iterator<Item = &'a Coordinate> + 'a>> for &'a Triangle {
    fn into(self) -> Box<dyn Iterator<Item = &'a Coordinate> + 'a> {
        let iter = self.points.iter();
        Box::new(iter)
    }
}

//[Line]への変換
impl Into<Box<dyn Iterator<Item = Line>>> for Triangle {
    fn into(self) -> Box<dyn Iterator<Item = Line>> {
        let iter = [
            Line::new([self.points[0], self.points[1]]),
            Line::new([self.points[1], self.points[2]]),
            Line::new([self.points[2], self.points[0]]),
        ]
        .into_iter();
        Box::new(iter)
    }
}

impl Into<Box<dyn Iterator<Item = Line>>> for &Triangle {
    fn into(self) -> Box<dyn Iterator<Item = Line>> {
        let iter = [
            Line::new([self.points[0], self.points[1]]),
            Line::new([self.points[1], self.points[2]]),
            Line::new([self.points[2], self.points[0]]),
        ]
        .into_iter();
        Box::new(iter)
    }
}
