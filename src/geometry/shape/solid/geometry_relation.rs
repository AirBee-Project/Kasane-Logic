#[allow(unused_imports)]
use alloc::boxed::Box;
#[allow(unused_imports)]
use alloc::rc::Rc;
#[allow(unused_imports)]
use alloc::string::{String, ToString};
#[allow(unused_imports)]
use alloc::vec::Vec;

use crate::{
    Coordinate, ExpandCoordinates, ExpandLines, ExpandPolygons, ExpandTriangles, Line, Polygon,
    Solid, Triangle,
};

impl ExpandCoordinates for Solid {
    fn expand_coordinates(&self) -> impl Iterator<Item = Coordinate> {
        self.polygons
            .iter()
            .flat_map(|polygon| polygon.expand_coordinates())
    }
}

impl ExpandLines for Solid {
    fn expand_lines(&self) -> impl Iterator<Item = Line> {
        self.polygons
            .iter()
            .flat_map(|polygon| polygon.expand_lines())
    }
}

impl ExpandTriangles for Solid {
    fn expand_triangles(&self) -> impl Iterator<Item = Triangle> {
        self.polygons
            .iter()
            .flat_map(|polygon| polygon.expand_triangles())
    }
}

impl ExpandPolygons for Solid {
    fn expand_polygons(&self) -> impl Iterator<Item = Polygon> {
        self.polygons.clone().into_iter()
    }
}
