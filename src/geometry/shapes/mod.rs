//! Shape モジュールの相互関係の概要。
//!
//! - `Line` は 2 つの `Coordinate` で表される。
//! - `Triangle` は 3 本の `Line`（または 3 つの `Coordinate`）で表される。
//! - `Polygon` は複数の `Triangle` に分解できる。
//! - `Solid` は複数の `Polygon` で構成される。
//! - `Sphere` は中心点と半径を持つ独立した Shape として扱う。
//!
//! 詳細な図と説明は次のドキュメントを参照:
//! [docs/geometry-relation.md](https://github.com/AirBee-Project/Kasane-Logic/blob/main/docs/geometry-relation.md)

pub mod line;
pub mod polygon;
pub mod solid;
pub mod sphere;
pub mod traits;
pub mod triangle;
