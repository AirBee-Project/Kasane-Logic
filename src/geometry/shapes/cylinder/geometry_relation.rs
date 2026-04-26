use crate::{
    Coordinate, Cylinder, Ecef, IntoCoordinates, IntoLines, IntoSolids, IntoTriangles, Line,
    Polygon, Solid, Triangle,SpatialVector
};

impl IntoSolids for Cylinder {
    fn into_solids(self) -> impl Iterator<Item = Solid> {
        let vec_n: [SpatialVector; 2] = self.points.map(|p| p.into());
        vec_n.iter().cloned()
    }
    fn iter_solids(&self) -> impl Iterator<Item = Solid> {
        todo!()
    }
}
