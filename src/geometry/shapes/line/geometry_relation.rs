use crate::{Coordinate, Line};

//[Coordinate]への変換
impl From<Line> for Box<dyn Iterator<Item = Coordinate>> {
    fn from(val: Line) -> Self {
        let iter = val.points.into_iter();
        Box::new(iter)
    }
}

impl From<&Line> for Box<dyn Iterator<Item = Coordinate>> {
    fn from(val: &Line) -> Self {
        let iter = val.points.into_iter();
        Box::new(iter)
    }
}

impl<'a> From<&'a Line> for Box<dyn Iterator<Item = &'a Coordinate> + 'a> {
    fn from(val: &'a Line) -> Self {
        let iter = val.points.iter();
        Box::new(iter)
    }
}
