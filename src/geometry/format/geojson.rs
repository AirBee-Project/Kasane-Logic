use ::geojson::{Feature, GeoJson, Geometry, GeometryValue, Position};
use alloc::vec::Vec;
use hashbrown::HashSet;

use crate::{
    Coordinate, CoverSingleIds, Epsilon, Error, GeometryError, Line, Point, Polygon, SingleId,
    SpatialIdMap,
};

/// Geometry だけの GeoJSON で、ID と組にする Feature。
static EMPTY_FEATURE: Feature = Feature {
    bbox: None,
    geometry: None,
    id: None,
    properties: None,
    foreign_members: None,
};

/// 許容誤差を指定して GeoJSON または Geometry を空間IDに変換するための拡張Trait。
pub trait GeoJsonCover {
    /// ID と組にする値の元となる型。
    type Value;

    /// 許容誤差を指定して、図形が覆う [`SingleId`] を `f` で変換した値と組にして返す。
    fn cover_single_ids_with_epsilon<'a, V, F>(
        &'a self,
        z: impl Into<u8>,
        epsilon: Epsilon,
        f: F,
    ) -> Result<impl Iterator<Item = (SingleId, V)> + 'a, Error>
    where
        V: Clone + 'a,
        F: FnMut(&'a Self::Value) -> V + 'a;

    /// 許容誤差を指定して、対象の図形が覆う [`SingleId`] を返す。
    fn cover_single_ids_epsilon(
        &self,
        z: impl Into<u8>,
        epsilon: Epsilon,
    ) -> Result<impl Iterator<Item = SingleId>, Error> {
        Ok(self
            .cover_single_ids_with_epsilon(z, epsilon, |_| ())?
            .map(|(id, _)| id))
    }

    /// 許容誤差を指定して、直接 [`SpatialIdMap`] を構築する。
    fn into_map_with_epsilon<V, F>(
        self,
        z: impl Into<u8>,
        epsilon: Epsilon,
        f: F,
    ) -> Result<SpatialIdMap<V>, Error>
    where
        Self: Sized,
        V: PartialEq + Clone + Send + Sync,
        F: FnMut(&Self::Value) -> V;
}

/// GeoJSON の全ジオメトリが覆う空間IDを求める。
///
/// - Position は `[経度, 緯度, 高度]` として読み、高度が省略された場合は 0 m とする。
/// - Polygon の穴は、穴の内部を除き、穴の境界線は含める。
/// - `geometry` が `null` の Feature は無視する。
/// - [`CoverSingleIds::cover_single_ids_with`] ではデフォルト許容誤差（[`Epsilon::DEFAULT`]）を使用し、空間IDへの展開前に Feature 単位で値を評価する。
/// - [`CoverSingleIds::into_map`] では直接 [`SpatialIdMap`] を構築する。
///   複数の Feature が同じ ID を覆う場合は、後の Feature の値が残る。
/// - 許容誤差をカスタマイズする場合は [`GeoJsonCover`] の各メソッドを使用する。
///
/// # 動作例
/// ```
/// # use spatial_id::geometry::CoverSingleIds;
/// let geojson: geojson::GeoJson = r#"{
///     "type": "FeatureCollection",
///     "features": [{
///         "type": "Feature",
///         "properties": { "name": "A" },
///         "geometry": { "type": "Point", "coordinates": [139.0, 35.0] }
///     }]
/// }"#.parse().unwrap();
///
/// let map = geojson.into_map(20, |feature| {
///     feature.property("name").unwrap().as_str().unwrap().to_owned()
/// }).unwrap();
/// assert!(map.iter().all(|(_, v)| v == "A"));
/// ```
impl CoverSingleIds for GeoJson {
    type Value = Feature;

    fn cover_single_ids_with<'a, V, F>(
        &'a self,
        z: impl Into<u8>,
        f: F,
    ) -> Result<impl Iterator<Item = (SingleId, V)> + 'a, Error>
    where
        V: Clone + 'a,
        F: FnMut(&'a Self::Value) -> V + 'a,
    {
        self.cover_single_ids_with_epsilon(z, Epsilon::DEFAULT, f)
    }

    fn into_map<V, F>(self, z: impl Into<u8>, f: F) -> Result<SpatialIdMap<V>, Error>
    where
        V: PartialEq + Clone + Send + Sync,
        F: FnMut(&Self::Value) -> V,
    {
        self.into_map_with_epsilon(z, Epsilon::DEFAULT, f)
    }
}

impl GeoJsonCover for GeoJson {
    type Value = Feature;

    fn cover_single_ids_with_epsilon<'a, V, F>(
        &'a self,
        z: impl Into<u8>,
        epsilon: Epsilon,
        mut f: F,
    ) -> Result<impl Iterator<Item = (SingleId, V)> + 'a, Error>
    where
        V: Clone + 'a,
        F: FnMut(&'a Self::Value) -> V + 'a,
    {
        let z = z.into();
        let mut pairs = Vec::new();
        match self {
            GeoJson::Geometry(geometry) => {
                let value = f(&EMPTY_FEATURE);
                for id in geometry.cover_single_ids_epsilon(z, epsilon)? {
                    pairs.push((id, value.clone()));
                }
            }
            GeoJson::Feature(feature) => {
                if let Some(geometry) = &feature.geometry {
                    let value = f(feature);
                    for id in geometry.cover_single_ids_epsilon(z, epsilon)? {
                        pairs.push((id, value.clone()));
                    }
                }
            }
            GeoJson::FeatureCollection(collection) => {
                for feature in &collection.features {
                    if let Some(geometry) = &feature.geometry {
                        let value = f(feature);
                        for id in geometry.cover_single_ids_epsilon(z, epsilon)? {
                            pairs.push((id, value.clone()));
                        }
                    }
                }
            }
        }
        Ok(pairs.into_iter())
    }

    fn into_map_with_epsilon<V, F>(
        self,
        z: impl Into<u8>,
        epsilon: Epsilon,
        f: F,
    ) -> Result<SpatialIdMap<V>, Error>
    where
        V: PartialEq + Clone + Send + Sync,
        F: FnMut(&Self::Value) -> V,
    {
        to_feature_collection_map(self, z.into(), epsilon, f)
    }
}

#[cfg(not(feature = "rayon"))]
fn to_feature_collection_map<V, F>(
    geojson: GeoJson,
    z: u8,
    epsilon: Epsilon,
    mut f: F,
) -> Result<SpatialIdMap<V>, Error>
where
    V: PartialEq + Clone,
    F: FnMut(&Feature) -> V,
{
    let mut map = SpatialIdMap::new();
    match geojson {
        GeoJson::Geometry(geometry) => {
            let value = f(&EMPTY_FEATURE);
            for id in geometry.cover_single_ids_epsilon(z, epsilon)? {
                map.insert(id, value.clone());
            }
        }
        GeoJson::Feature(feature) => {
            let value = f(&feature);
            if let Some(geometry) = feature.geometry {
                for id in geometry.cover_single_ids_epsilon(z, epsilon)? {
                    map.insert(id, value.clone());
                }
            }
        }
        GeoJson::FeatureCollection(collection) => {
            for feature in collection.features {
                let value = f(&feature);
                let geometry = match feature.geometry {
                    Some(g) => g,
                    None => continue,
                };
                for id in geometry.cover_single_ids_epsilon(z, epsilon)? {
                    map.insert(id, value.clone());
                }
            }
        }
    }
    Ok(map)
}

#[cfg(feature = "rayon")]
fn to_feature_collection_map<V, F>(
    geojson: GeoJson,
    z: u8,
    epsilon: Epsilon,
    mut f: F,
) -> Result<SpatialIdMap<V>, Error>
where
    V: PartialEq + Clone + Send + Sync,
    F: FnMut(&Feature) -> V,
{
    use rayon::prelude::*;
    let later_wins = |a: SpatialIdMap<V>, b: SpatialIdMap<V>| SpatialIdMap {
        inner: b.inner.union(&a.inner),
    };

    match geojson {
        GeoJson::Geometry(geometry) => {
            let value = f(&EMPTY_FEATURE);
            let mut map = SpatialIdMap::new();
            for id in geometry.cover_single_ids_epsilon(z, epsilon)? {
                map.insert(id, value.clone());
            }
            Ok(map)
        }
        GeoJson::Feature(feature) => {
            let mut map = SpatialIdMap::new();
            let value = f(&feature);
            if let Some(geometry) = feature.geometry {
                for id in geometry.cover_single_ids_epsilon(z, epsilon)? {
                    map.insert(id, value.clone());
                }
            }
            Ok(map)
        }
        GeoJson::FeatureCollection(collection) => {
            let pairs: Vec<(Geometry, V)> = collection
                .features
                .into_iter()
                .filter_map(|feature| {
                    let value = f(&feature);
                    let geometry = feature.geometry?;
                    Some((geometry, value))
                })
                .collect();

            let map = pairs
                .into_par_iter()
                .map(|(geometry, value)| {
                    let mut map = SpatialIdMap::new();
                    if let Ok(ids) = geometry.cover_single_ids_epsilon(z, epsilon) {
                        for id in ids {
                            map.insert(id, value.clone());
                        }
                    }
                    map
                })
                .reduce(SpatialIdMap::new, later_wins);

            Ok(map)
        }
    }
}

impl CoverSingleIds for Geometry {
    type Value = ();

    fn cover_single_ids_with<'a, V, F>(
        &'a self,
        z: impl Into<u8>,
        f: F,
    ) -> Result<impl Iterator<Item = (SingleId, V)> + 'a, Error>
    where
        V: Clone + 'a,
        F: FnMut(&'a Self::Value) -> V + 'a,
    {
        self.cover_single_ids_with_epsilon(z, Epsilon::DEFAULT, f)
    }
}

impl GeoJsonCover for Geometry {
    type Value = ();

    fn cover_single_ids_with_epsilon<'a, V, F>(
        &'a self,
        z: impl Into<u8>,
        epsilon: Epsilon,
        mut f: F,
    ) -> Result<impl Iterator<Item = (SingleId, V)> + 'a, Error>
    where
        V: Clone + 'a,
        F: FnMut(&'a Self::Value) -> V + 'a,
    {
        let value = f(&());
        let z = z.into();
        let mut ids = HashSet::new();
        match &self.value {
            GeometryValue::Point { coordinates } => {
                ids.insert(Coordinate::try_from(coordinates)?.single_id(z)?);
            }
            GeometryValue::MultiPoint { coordinates } => {
                for position in coordinates {
                    ids.insert(Coordinate::try_from(position)?.single_id(z)?);
                }
            }
            GeometryValue::LineString { coordinates } => {
                let coords: Vec<Coordinate> = coordinates
                    .iter()
                    .map(Coordinate::try_from)
                    .collect::<Result<_, _>>()?;
                ids.extend(Line::cover_polyline(&coords, z)?);
            }
            GeometryValue::MultiLineString { coordinates } => {
                for line_string in coordinates {
                    let coords: Vec<Coordinate> = line_string
                        .iter()
                        .map(Coordinate::try_from)
                        .collect::<Result<_, _>>()?;
                    ids.extend(Line::cover_polyline(&coords, z)?);
                }
            }
            GeometryValue::Polygon { coordinates } => {
                let polygon =
                    Polygon::try_from_geojson_rings_with_epsilon(coordinates.as_slice(), epsilon)?;
                ids.extend(polygon.cover_single_ids(z)?);
            }
            GeometryValue::MultiPolygon { coordinates } => {
                for rings in coordinates {
                    let polygon =
                        Polygon::try_from_geojson_rings_with_epsilon(rings.as_slice(), epsilon)?;
                    ids.extend(polygon.cover_single_ids(z)?);
                }
            }
            GeometryValue::GeometryCollection { geometries } => {
                for geometry in geometries {
                    ids.extend(geometry.cover_single_ids_epsilon(z, epsilon)?);
                }
            }
        }
        Ok(ids.into_iter().map(move |id| (id, value.clone())))
    }

    fn into_map_with_epsilon<V, F>(
        self,
        z: impl Into<u8>,
        epsilon: Epsilon,
        f: F,
    ) -> Result<SpatialIdMap<V>, Error>
    where
        Self: Sized,
        V: PartialEq + Clone + Send + Sync,
        F: FnMut(&Self::Value) -> V,
    {
        let mut map = SpatialIdMap::new();
        for (id, val) in self.cover_single_ids_with_epsilon(z, epsilon, f)? {
            map.insert(id, val);
        }
        Ok(map)
    }
}

impl Polygon {
    /// GeoJSON のリング座標列からデフォルト許容誤差（[`Epsilon::DEFAULT`]）を使用して新しい [`Polygon`] を作成する。
    pub fn try_from_geojson_rings(rings: &[Vec<Position>]) -> Result<Self, Error> {
        Self::try_from_geojson_rings_with_epsilon(rings, Epsilon::DEFAULT)
    }

    /// GeoJSON のリング座標列と指定された許容誤差から新しい [`Polygon`] を作成する。
    pub fn try_from_geojson_rings_with_epsilon(
        rings: &[Vec<Position>],
        epsilon: Epsilon,
    ) -> Result<Self, Error> {
        let exterior = match rings.first() {
            Some(ring) => ring
                .iter()
                .map(Coordinate::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            None => return Ok(Polygon::new(Vec::new())),
        };
        let holes = rings
            .iter()
            .skip(1)
            .map(|ring| {
                ring.iter()
                    .map(Coordinate::try_from)
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Polygon::with_holes_and_epsilon(exterior, holes, epsilon))
    }
}

impl TryFrom<&[Vec<Position>]> for Polygon {
    type Error = Error;

    fn try_from(rings: &[Vec<Position>]) -> Result<Self, Self::Error> {
        Self::try_from_geojson_rings(rings)
    }
}

impl TryFrom<&Position> for Coordinate {
    type Error = Error;

    fn try_from(position: &Position) -> Result<Self, Self::Error> {
        if position.len() < 2 {
            return Err(GeometryError::PositionTooShort {
                len: position.len(),
            }
            .into());
        }
        let lon = position[0];
        let lat = position[1];
        let alt = if position.len() > 2 { position[2] } else { 0.0 };
        Coordinate::new(lat, lon, alt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geojson_polygon_epsilon() {
        // 経度、緯度、高度
        let p0 = Position::from([139.0, 35.0, 0.0]);
        let p1 = Position::from([139.001, 35.0, 0.0]);
        let p2 = Position::from([139.0, 35.001, 0.0]);
        let p_close = Position::from([139.0, 35.001000001, 0.0]);

        let rings: Vec<Vec<Position>> = vec![vec![p0, p1, p2, p_close]];

        // 許容誤差大 (1.0m): p_close が p2 に統合されて3頂点
        let poly_coarse = Polygon::try_from_geojson_rings_with_epsilon(
            rings.as_slice(),
            Epsilon::from_meters(1.0).unwrap(),
        )
        .unwrap();
        assert_eq!(poly_coarse.vertices().len(), 3);

        // 許容誤差小 (1e-6m): p_close は統合されず4頂点
        let poly_fine = Polygon::try_from_geojson_rings_with_epsilon(
            rings.as_slice(),
            Epsilon::from_meters(1e-6).unwrap(),
        )
        .unwrap();
        assert_eq!(poly_fine.vertices().len(), 4);

        // TryFrom ではデフォルト許容誤差（1e-10m）が使用され4頂点
        let poly_default = Polygon::try_from(rings.as_slice()).unwrap();
        assert_eq!(poly_default.vertices().len(), 4);
    }

    #[test]
    fn test_geojson_cover_with_epsilon() {
        let geojson_str = r#"{
            "type": "FeatureCollection",
            "features": [{
                "type": "Feature",
                "properties": { "name": "test" },
                "geometry": {
                    "type": "Polygon",
                    "coordinates": [
                        [
                            [139.760, 35.680, 10.0],
                            [139.760, 35.690, 10.0],
                            [139.770, 35.690, 10.0],
                            [139.770, 35.680, 10.0],
                            [139.760, 35.680, 10.0]
                        ]
                    ]
                }
            }]
        }"#;

        let geojson: GeoJson = geojson_str.parse().unwrap();
        let map = geojson
            .into_map_with_epsilon(18, Epsilon::from_meters(1.0).unwrap(), |f| {
                f.property("name").unwrap().as_str().unwrap().to_owned()
            })
            .unwrap();
        assert!(!map.is_empty());
    }
}
