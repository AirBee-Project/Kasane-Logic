use crate::{Coordinate, Error, Line, Polygon, RangeId, SingleId, Triangle};

/// 現実空間の図形に対して共通で定義することができる性質
///
/// <https://github.com/AirBee-Project/Kasane-Logic/blob/main/docs/geometry-relation.md>
pub trait Shape {
    fn center(&self) -> Coordinate;

    /// あるズームレベルの[SingleId]を出力する。
    fn single_ids(&self, z: u8) -> Result<impl Iterator<Item = SingleId>, Error>;

    /// あるズームレベルの[RangeId]を出力する。
    fn range_ids(&self, z: u8) -> Result<impl Iterator<Item = RangeId>, Error>;

    /// 最小の個数の[SingleId]で出力する。
    ///
    /// 最小の個数を保証する。
    fn optimze_single_ids(&self, z: u8) -> Result<impl Iterator<Item = SingleId>, Error> {
        self.single_ids(z)
    }

    /// 最小の個数の[RangeId]で出力する。
    ///
    /// 最小の個数を保証する。
    fn optimze_range_ids(&self, z: u8) -> Result<impl Iterator<Item = RangeId>, Error> {
        self.range_ids(z)
    }
}

/// [Coordinate] の集合へ分解可能であることを示す
///
/// <https://github.com/AirBee-Project/Kasane-Logic/blob/main/docs/geometry-relation.md>
///
/// # Examples
/// ```
/// use kasane_logic::{Coordinate, IntoCoordinates, Line};
///
/// let p0 = Coordinate::new(35.0, 139.0, 10.0).unwrap();
/// let p1 = Coordinate::new(35.0001, 139.0001, 11.0).unwrap();
/// let line = Line::new([p0, p1]);
///
/// let coords: Vec<Coordinate> = line.iter_coordinates().collect();
/// assert_eq!(coords.len(), 2);
/// ```
pub trait IntoCoordinates {
    fn into_coordinates(self) -> impl Iterator<Item = Coordinate>;
    fn iter_coordinates(&self) -> impl Iterator<Item = Coordinate>;
}

/// [Line] の集合へ分解可能であることを示す
///
/// <https://github.com/AirBee-Project/Kasane-Logic/blob/main/docs/geometry-relation.md>
///
/// # Examples
/// ```
/// use kasane_logic::{Coordinate, IntoLines, Line, Triangle};
///
/// let p0 = Coordinate::new(35.0, 139.0, 10.0).unwrap();
/// let p1 = Coordinate::new(35.0002, 139.0, 10.0).unwrap();
/// let p2 = Coordinate::new(35.0, 139.0002, 10.0).unwrap();
/// let tri = Triangle::new([p0, p1, p2]);
///
/// let lines: Vec<Line> = tri.into_lines().collect();
/// assert_eq!(lines.len(), 3);
/// ```
pub trait IntoLines {
    fn into_lines(self) -> impl Iterator<Item = Line>;
    fn iter_lines(&self) -> impl Iterator<Item = Line>;
}

/// [Triangle] の集合へ分解可能であることを示す
///
/// <https://github.com/AirBee-Project/Kasane-Logic/blob/main/docs/geometry-relation.md>
///
/// # Examples
/// ```
/// use kasane_logic::{Coordinate, IntoTriangles, Polygon, Triangle};
///
/// let p0 = Coordinate::new(35.0, 139.0, 10.0).unwrap();
/// let p1 = Coordinate::new(35.0003, 139.0, 10.0).unwrap();
/// let p2 = Coordinate::new(35.0003, 139.0003, 10.0).unwrap();
/// let p3 = Coordinate::new(35.0, 139.0003, 10.0).unwrap();
/// let polygon = Polygon::new(vec![p0, p1, p2, p3], 0.01);
///
/// let triangles: Vec<Triangle> = polygon.iter_triangles().collect();
/// assert_eq!(triangles.len(), 2);
/// ```
pub trait IntoTriangles {
    fn into_triangles(self) -> impl Iterator<Item = Triangle>;
    fn iter_triangles(&self) -> impl Iterator<Item = Triangle>;
}

/// [Polygon] の集合へ分解可能であることを示す
///
/// <https://github.com/AirBee-Project/Kasane-Logic/blob/main/docs/geometry-relation.md>
///
/// # Examples
/// ```
/// use kasane_logic::{Coordinate, IntoPolygons, Polygon, Solid};
///
/// let p0 = Coordinate::new(35.0, 139.0, 10.0).unwrap();
/// let p1 = Coordinate::new(35.0003, 139.0, 10.0).unwrap();
/// let p2 = Coordinate::new(35.0, 139.0003, 10.0).unwrap();
/// let p3 = Coordinate::new(35.0002, 139.0002, 10.3).unwrap();
///
/// let surfaces = vec![
///     vec![p0, p1, p2],
///     vec![p0, p3, p1],
///     vec![p1, p3, p2],
///     vec![p2, p3, p0],
/// ];
///
/// let solid = Solid::new(surfaces, 0.01).unwrap();
/// let polygons: Vec<Polygon> = solid.into_polygons().collect();
/// assert_eq!(polygons.len(), 4);
/// ```
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
