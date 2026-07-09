use crate::{
    Coordinate, ExpandCoordinates, ExpandLines, ExpandPolygons, ExpandTriangles, Line, Polygon,
    Solid, Triangle,
};

impl ExpandCoordinates for Solid {
    fn expand_coordinates(&self) -> impl Iterator<Item = Coordinate> {
        self.polygons
            .iter()
            .flat_map(super::super::traits::ExpandCoordinates::expand_coordinates)
    }
}

impl ExpandLines for Solid {
    fn expand_lines(&self) -> impl Iterator<Item = Line> {
        self.polygons
            .iter()
            .flat_map(super::super::traits::ExpandLines::expand_lines)
    }
}

impl ExpandTriangles for Solid {
    fn expand_triangles(&self) -> impl Iterator<Item = Triangle> {
        self.polygons
            .iter()
            .flat_map(super::super::traits::ExpandTriangles::expand_triangles)
    }
}

impl ExpandPolygons for Solid {
    fn expand_polygons(&self) -> impl Iterator<Item = Polygon> {
        self.polygons.clone().into_iter()
    }
}
