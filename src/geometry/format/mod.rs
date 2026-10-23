//! 地理空間データのフォーマット。

#[cfg(feature = "std")]
mod geojson;
pub(crate) mod regionmesh;

pub use regionmesh::RegionMesh;
