use crate::{Coordinate, Line, Polygon, Triangle};

/// [Coordinate] の集合へ分解可能であることを示す
pub trait IntoCoordinates {
    fn into_coordinates(self) -> impl Iterator<Item = Coordinate>;
    fn iter_coordinates(&self) -> impl Iterator<Item = Coordinate>;
}

/// [Line] の集合へ分解可能であることを示す
pub trait IntoLines {
    fn into_lines(self) -> impl Iterator<Item = Line>;
    fn iter_lines(&self) -> impl Iterator<Item = Line>;
}

/// [Triangle] の集合へ分解可能であることを示す
pub trait IntoTriangles {
    fn into_triangles(self) -> impl Iterator<Item = Triangle>;
    fn iter_triangles(&self) -> impl Iterator<Item = Triangle>;
}

/// [Polygon] の集合へ分解可能であることを示す
pub trait IntoPolygons {
    fn into_polygons(self) -> impl Iterator<Item = Polygon>;
    fn iter_polygons(&self) -> impl Iterator<Item = Polygon>;
}

// Coordinate への分解
impl<G> IntoCoordinates for G
where
    G: Into<Box<dyn Iterator<Item = Coordinate>>>,
    for<'a> &'a G: Into<Box<dyn Iterator<Item = Coordinate>>>,
{
    fn into_coordinates(self) -> impl Iterator<Item = Coordinate> {
        self.into()
    }
    fn iter_coordinates(&self) -> impl Iterator<Item = Coordinate> {
        self.into()
    }
}

// Line への分解
impl<G> IntoLines for G
where
    G: Into<Box<dyn Iterator<Item = Line>>>,
    for<'a> &'a G: Into<Box<dyn Iterator<Item = Line>>>,
{
    fn into_lines(self) -> impl Iterator<Item = Line> {
        self.into()
    }
    fn iter_lines(&self) -> impl Iterator<Item = Line> {
        self.into()
    }
}

// Triangle への分解
impl<G> IntoTriangles for G
where
    G: Into<Box<dyn Iterator<Item = Triangle>>>,
    for<'a> &'a G: Into<Box<dyn Iterator<Item = Triangle>>>,
{
    fn into_triangles(self) -> impl Iterator<Item = Triangle> {
        self.into()
    }
    fn iter_triangles(&self) -> impl Iterator<Item = Triangle> {
        self.into()
    }
}

// Polygon への分解
impl<G> IntoPolygons for G
where
    G: Into<Box<dyn Iterator<Item = Polygon>>>,
    for<'a> &'a G: Into<Box<dyn Iterator<Item = Polygon>>>,
{
    fn into_polygons(self) -> impl Iterator<Item = Polygon> {
        self.into()
    }
    fn iter_polygons(&self) -> impl Iterator<Item = Polygon> {
        self.into()
    }
}
