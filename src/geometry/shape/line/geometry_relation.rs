#[allow(unused_imports)]
use alloc::boxed::Box;
#[allow(unused_imports)]
use alloc::rc::Rc;
#[allow(unused_imports)]
use alloc::string::{String, ToString};
#[allow(unused_imports)]
use alloc::vec::Vec;

use crate::{Coordinate, ExpandCoordinates, Line};

impl ExpandCoordinates for Line {
    fn expand_coordinates(&self) -> impl Iterator<Item = Coordinate> {
        self.points.into_iter()
    }
}
