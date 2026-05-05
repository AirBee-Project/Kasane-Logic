use crate::{
    Coordinate, IterCoordinates, IterLines, IterPolygons, IterTriangles, Line, Polygon, Solid,
    Triangle,
};

impl IterCoordinates for Solid {
    fn iter_coordinates(&self) -> impl Iterator<Item = Coordinate> {
        self.polygons
            .iter()
            .flat_map(|polygon| polygon.iter_coordinates())
    }
}

impl IterLines for Solid {
    fn iter_lines(&self) -> impl Iterator<Item = Line> {
        self.polygons
            .iter()
            .flat_map(|polygon| polygon.iter_lines())
    }
}

impl IterTriangles for Solid {
    fn iter_triangles(&self) -> impl Iterator<Item = Triangle> {
        self.polygons
            .iter()
            .flat_map(|polygon| polygon.iter_triangles())
    }
}

impl IterPolygons for Solid {
    fn iter_polygons(&self) -> impl Iterator<Item = Polygon> {
        self.polygons.clone().into_iter()
    }
}
