use crate::{Coordinate, Line, Triangle};

//[Coordinate]への変換
impl From<Triangle> for Box<dyn Iterator<Item = Coordinate>> {
    fn from(val: Triangle) -> Self {
        let iter = val.points.into_iter();
        Box::new(iter)
    }
}

impl From<&Triangle> for Box<dyn Iterator<Item = Coordinate>> {
    fn from(val: &Triangle) -> Self {
        let iter = val.points.into_iter();
        Box::new(iter)
    }
}

impl<'a> From<&'a Triangle> for Box<dyn Iterator<Item = &'a Coordinate> + 'a> {
    fn from(val: &'a Triangle) -> Self {
        let iter = val.points.iter();
        Box::new(iter)
    }
}

//[Line]への変換
impl From<Triangle> for Box<dyn Iterator<Item = Line>> {
    fn from(val: Triangle) -> Self {
        let iter = [
            Line::new([val.points[0], val.points[1]]),
            Line::new([val.points[1], val.points[2]]),
            Line::new([val.points[2], val.points[0]]),
        ]
        .into_iter();
        Box::new(iter)
    }
}

impl From<&Triangle> for Box<dyn Iterator<Item = Line>> {
    fn from(val: &Triangle) -> Self {
        let iter = [
            Line::new([val.points[0], val.points[1]]),
            Line::new([val.points[1], val.points[2]]),
            Line::new([val.points[2], val.points[0]]),
        ]
        .into_iter();
        Box::new(iter)
    }
}
