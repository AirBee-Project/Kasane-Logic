use ::geojson::GeoJson;

use crate::CoverSingleIds;

#[test]
fn cover_map_returns_error_of_invalid_feature() {
    let geojson: GeoJson = r#"{
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "properties": null,
                "geometry": { "type": "Point", "coordinates": [139.0, 35.0] }
            },
            {
                "type": "Feature",
                "properties": null,
                "geometry": { "type": "Point", "coordinates": [139.0, 100.0] }
            }
        ]
    }"#
    .parse()
    .unwrap();

    assert!(geojson.cover_map(20, |_| ()).is_err());
}
