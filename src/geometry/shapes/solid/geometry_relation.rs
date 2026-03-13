use crate::{Coordinate, Line, Polygon, Solid, Triangle};

//[Coordinate]への変換
impl Into<Box<dyn Iterator<Item = Coordinate>>> for Solid {
    fn into(self) -> Box<dyn Iterator<Item = Coordinate>> {
        (&self).into()
    }
}

impl Into<Box<dyn Iterator<Item = Coordinate>>> for &Solid {
    fn into(self) -> Box<dyn Iterator<Item = Coordinate>> {
        let coords_iter = self.polygons.clone().into_iter().flat_map(|polygon| {
            let coords: Box<dyn Iterator<Item = Coordinate>> = polygon.into();
            coords
        });

        Box::new(coords_iter)
    }
}

impl<'a> Into<Box<dyn Iterator<Item = &'a Coordinate> + 'a>> for &'a Solid {
    fn into(self) -> Box<dyn Iterator<Item = &'a Coordinate> + 'a> {
        let coords_iter = self.polygons.iter().flat_map(|polygon| {
            let coords: Box<dyn Iterator<Item = &'a Coordinate> + 'a> = polygon.into();
            coords
        });
        Box::new(coords_iter)
    }
}

//[Line]への変換
impl Into<Box<dyn Iterator<Item = Line>>> for Solid {
    fn into(self) -> Box<dyn Iterator<Item = Line>> {
        let iter = self.polygons.into_iter().flat_map(|polygon| {
            let triangles: Box<dyn Iterator<Item = Triangle>> = polygon.into();
            triangles.flat_map(|triangle| {
                let lines: Box<dyn Iterator<Item = Line>> = triangle.into();
                lines
            })
        });
        Box::new(iter)
    }
}

impl Into<Box<dyn Iterator<Item = Line>>> for &Solid {
    fn into(self) -> Box<dyn Iterator<Item = Line>> {
        let copy = self.clone();
        copy.into()
    }
}

//[Triangle]への変換
impl Into<Box<dyn Iterator<Item = Triangle>>> for Solid {
    fn into(self) -> Box<dyn Iterator<Item = Triangle>> {
        let iter = self.polygons.into_iter().flat_map(|polygon| {
            let triangles: Box<dyn Iterator<Item = Triangle>> = polygon.into();
            triangles.into_iter()
        });
        Box::new(iter)
    }
}

impl Into<Box<dyn Iterator<Item = Triangle>>> for &Solid {
    fn into(self) -> Box<dyn Iterator<Item = Triangle>> {
        let copy = self.clone();
        copy.into()
    }
}

//[Triangle]への変換
impl Into<Box<dyn Iterator<Item = Polygon>>> for Solid {
    fn into(self) -> Box<dyn Iterator<Item = Polygon>> {
        let iter = self.polygons.into_iter();
        Box::new(iter)
    }
}

impl Into<Box<dyn Iterator<Item = Polygon>>> for &Solid {
    fn into(self) -> Box<dyn Iterator<Item = Polygon>> {
        let copy = self.clone();
        copy.into()
    }
}
