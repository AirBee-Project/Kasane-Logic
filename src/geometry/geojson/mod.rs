use alloc::vec::Vec;
use geojson::{Geometry, Value};
use hashbrown::HashSet;

use crate::geometry::traits::CoverSingleIds;
use crate::{Coordinate, Error, Line, Polygon, SingleId};

/// GeoJsonのジオメトリとユーザー定義の値を保持する構造体。
#[derive(Debug, Clone)]
pub struct GeoJsonFeature<V> {
    pub geometry: Geometry,
    pub value: V,
    pub epsilon: f64,
    pub default_altitude: f64,
}

impl<V> GeoJsonFeature<V> {
    /// 新しいGeoJsonFeatureを作成する。
    pub fn new(geometry: Geometry, value: V, epsilon: f64, default_altitude: f64) -> Self {
        Self {
            geometry,
            value,
            epsilon,
            default_altitude,
        }
    }

    /// 空間IDに分解する前に、事前に Value を変換しておくことで計算コストを抑える。
    pub fn map_value<U, F: FnOnce(V) -> U>(self, f: F) -> GeoJsonFeature<U> {
        GeoJsonFeature {
            geometry: self.geometry,
            epsilon: self.epsilon,
            default_altitude: self.default_altitude,
            value: f(self.value),
        }
    }
}

// 2Dまたは3D座標をCoordinateに変換するヘルパー関数
fn parse_point(p: &Vec<f64>, default_alt: f64) -> Option<Coordinate> {
    let lon = *p.first()?;
    let lat = *p.get(1)?;
    let alt = p.get(2).copied().unwrap_or(default_alt);
    Coordinate::new(lat, lon, alt).ok()
}

impl<V: Clone> CoverSingleIds for GeoJsonFeature<V> {
    type Value = V;

    fn cover_single_ids_with(
        &self,
        z: u8,
    ) -> Result<impl Iterator<Item = (SingleId, Self::Value)>, Error> {
        let mut ids = HashSet::new();
        match &self.geometry.value {
            Value::Point(p) => {
                if let Some(coord) = parse_point(p, self.default_altitude) {
                    ids.extend(coord.cover_single_ids(z)?);
                }
            }
            Value::MultiPoint(mp) => {
                for p in mp {
                    if let Some(coord) = parse_point(p, self.default_altitude) {
                        ids.extend(coord.cover_single_ids(z)?);
                    }
                }
            }
            Value::LineString(ls) => {
                let coords: Vec<_> = ls
                    .iter()
                    .filter_map(|p| parse_point(p, self.default_altitude))
                    .collect();
                for window in coords.windows(2) {
                    let line = Line::new([window[0], window[1]]);
                    ids.extend(line.cover_single_ids(z)?);
                }
            }
            Value::MultiLineString(mls) => {
                for ls in mls {
                    let coords: Vec<_> = ls
                        .iter()
                        .filter_map(|p| parse_point(p, self.default_altitude))
                        .collect();
                    for window in coords.windows(2) {
                        let line = Line::new([window[0], window[1]]);
                        ids.extend(line.cover_single_ids(z)?);
                    }
                }
            }
            Value::Polygon(poly) => {
                // Polygon is a list of rings. The first ring is the exterior ring.
                if let Some(exterior) = poly.first() {
                    let coords: Vec<_> = exterior
                        .iter()
                        .filter_map(|p| parse_point(p, self.default_altitude))
                        .collect();
                    if coords.len() >= 3 {
                        let polygon = Polygon::new(coords, self.epsilon);
                        ids.extend(polygon.cover_single_ids(z)?);
                    }
                }
            }
            Value::MultiPolygon(mpoly) => {
                for poly in mpoly {
                    if let Some(exterior) = poly.first() {
                        let coords: Vec<_> = exterior
                            .iter()
                            .filter_map(|p| parse_point(p, self.default_altitude))
                            .collect();
                        if coords.len() >= 3 {
                            let polygon = Polygon::new(coords, self.epsilon);
                            ids.extend(polygon.cover_single_ids(z)?);
                        }
                    }
                }
            }
            Value::GeometryCollection(gc) => {
                // Not supported natively to avoid recursion complexity here, 
                // but user can map GeometryCollection externally.
            }
        }

        let val = self.value.clone();
        Ok(ids.into_iter().map(move |id| (id, val.clone())))
    }
}
