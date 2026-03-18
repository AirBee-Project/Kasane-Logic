use crate::{Coordinate, Line, Polygon, Solid, Triangle};

//[Coordinate]への変換
impl From<Solid> for Box<dyn Iterator<Item = Coordinate>> {
    fn from(val: Solid) -> Self {
        (&val).into()
    }
}

impl From<&Solid> for Box<dyn Iterator<Item = Coordinate>> {
    fn from(val: &Solid) -> Self {
        let coords_iter = val.polygons.clone().into_iter().flat_map(|polygon| {
            let coords: Box<dyn Iterator<Item = Coordinate>> = polygon.into();
            coords
        });

        Box::new(coords_iter)
    }
}

impl<'a> From<&'a Solid> for Box<dyn Iterator<Item = &'a Coordinate> + 'a> {
    fn from(val: &'a Solid) -> Self {
        let coords_iter = val.polygons.iter().flat_map(|polygon| {
            let coords: Box<dyn Iterator<Item = &'a Coordinate> + 'a> = polygon.into();
            coords
        });
        Box::new(coords_iter)
    }
}

//[Line]への変換
impl From<Solid> for Box<dyn Iterator<Item = Line>> {
    fn from(val: Solid) -> Self {
        let iter = val.polygons.into_iter().flat_map(|polygon| {
            let triangles: Box<dyn Iterator<Item = Triangle>> = polygon.into();
            triangles.flat_map(|triangle| {
                let lines: Box<dyn Iterator<Item = Line>> = triangle.into();
                lines
            })
        });
        Box::new(iter)
    }
}

impl From<&Solid> for Box<dyn Iterator<Item = Line>> {
    fn from(val: &Solid) -> Self {
        let copy = val.clone();
        copy.into()
    }
}

//[Triangle]への変換
impl From<Solid> for Box<dyn Iterator<Item = Triangle>> {
    fn from(val: Solid) -> Self {
        let iter = val.polygons.into_iter().flat_map(|polygon| {
            let triangles: Box<dyn Iterator<Item = Triangle>> = polygon.into();
            triangles.into_iter()
        });
        Box::new(iter)
    }
}

impl From<&Solid> for Box<dyn Iterator<Item = Triangle>> {
    fn from(val: &Solid) -> Self {
        let copy = val.clone();
        copy.into()
    }
}

//[Triangle]への変換
impl From<Solid> for Box<dyn Iterator<Item = Polygon>> {
    fn from(val: Solid) -> Self {
        let iter = val.polygons.into_iter();
        Box::new(iter)
    }
}

impl From<&Solid> for Box<dyn Iterator<Item = Polygon>> {
    fn from(val: &Solid) -> Self {
        let copy = val.clone();
        copy.into()
    }
}
