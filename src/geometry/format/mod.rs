//! 地理空間データのフォーマット。

#[cfg(feature = "std")]
mod geojson;
pub(crate) mod regionmesh;

#[cfg(feature = "std")]
pub use geojson::GeoJsonCover;
pub use regionmesh::RegionMesh;
