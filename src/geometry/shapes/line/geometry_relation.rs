use crate::{Coordinate, Line};

//[Coordinate]への変換
impl Into<Box<dyn Iterator<Item = Coordinate>>> for Line {
    fn into(self) -> Box<dyn Iterator<Item = Coordinate>> {
        let iter = self.points.into_iter();
        Box::new(iter)
    }
}

impl Into<Box<dyn Iterator<Item = Coordinate>>> for &Line {
    fn into(self) -> Box<dyn Iterator<Item = Coordinate>> {
        let iter = self.points.into_iter();
        Box::new(iter)
    }
}

impl<'a> Into<Box<dyn Iterator<Item = &'a Coordinate> + 'a>> for &'a Line {
    fn into(self) -> Box<dyn Iterator<Item = &'a Coordinate> + 'a> {
        let iter = self.points.iter();
        Box::new(iter)
    }
}
