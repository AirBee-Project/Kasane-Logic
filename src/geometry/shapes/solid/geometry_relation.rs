use crate::{
    Coordinate, IntoCoordinates, IntoLines, IntoPolygons, IntoTriangles, Line, Polygon, Solid,
    Triangle,
};

impl IntoCoordinates for Solid {
    fn into_coordinates(self) -> impl Iterator<Item = Coordinate> {
        self.polygons
            .into_iter()
            .flat_map(|polygon| polygon.into_coordinates())
    }

    fn iter_coordinates(&self) -> impl Iterator<Item = Coordinate> {
        self.polygons
            .iter()
            .flat_map(|polygon| polygon.iter_coordinates())
    }
}

impl IntoLines for Solid {
    fn into_lines(self) -> impl Iterator<Item = Line> {
        self.polygons
            .into_iter()
            .flat_map(|polygon| polygon.into_lines())
    }

    fn iter_lines(&self) -> impl Iterator<Item = Line> {
        self.polygons
            .iter()
            .flat_map(|polygon| polygon.iter_lines())
    }
}

impl IntoTriangles for Solid {
    fn into_triangles(self) -> impl Iterator<Item = Triangle> {
        self.polygons
            .into_iter()
            .flat_map(|polygon| polygon.into_triangles())
    }

    fn iter_triangles(&self) -> impl Iterator<Item = Triangle> {
        self.polygons
            .iter()
            .flat_map(|polygon| polygon.iter_triangles())
    }
}

impl IntoPolygons for Solid {
    fn into_polygons(self) -> impl Iterator<Item = Polygon> {
        self.polygons.into_iter()
    }

    fn iter_polygons(&self) -> impl Iterator<Item = Polygon> {
        self.polygons.clone().into_iter()
    }
}
