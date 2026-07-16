use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::SpatialIdMap;

use super::super::json::{deserialize_with_values, serialize_with_values};

impl<V> Serialize for SpatialIdMap<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue + Serialize + PartialEq,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_with_values(self.iter(), serializer)
    }
}

impl<'de, V> Deserialize<'de> for SpatialIdMap<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue + Deserialize<'de> + Clone,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut map = Self::new();
        for (range_id, value) in deserialize_with_values(deserializer)? {
            map.insert(range_id, value);
        }
        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use crate::{SingleId, SpatialIdMap};
    use alloc::string::{String, ToString};

    #[test]
    fn serialize_lists_unique_values_and_refs() {
        let mut map = SpatialIdMap::<i32>::new();
        map.insert(SingleId::new(20, 0, 0, 0).unwrap(), 10);
        map.insert(SingleId::new(20, 1, 0, 0).unwrap(), 20);
        map.insert(SingleId::new(20, 2, 0, 0).unwrap(), 10); // 値 10 を再利用

        let json = serde_json::to_string(&map).unwrap();

        // 値は数値としてそのまま（クォートされない）、重複は1つに集約。
        assert!(json.contains("\"value\":[10,20]"));
        assert!(json.contains("\"ref\":0"));
        assert!(json.contains("\"ref\":1"));
        assert!(json.contains("\"z\":20"));
    }

    #[test]
    fn serialize_quotes_and_escapes_string_values() {
        let mut map = SpatialIdMap::<String>::new();
        map.insert(SingleId::new(20, 0, 0, 0).unwrap(), "a\"b".to_string());

        let json = serde_json::to_string(&map).unwrap();
        // 文字列値はクォートされ、" がエスケープされる。
        assert!(json.contains("\"value\":[\"a\\\"b\"]"));
    }

    #[test]
    fn serialize_empty_map_has_empty_value_and_ids() {
        let map = SpatialIdMap::<i32>::new();
        let json = serde_json::to_string(&map).unwrap();
        assert!(json.contains("\"value\":[]"));
        assert!(json.contains("\"ids\":[]"));
    }

    #[test]
    fn deserialize_round_trips_values_and_ids() {
        let mut map = SpatialIdMap::<i32>::new();
        map.insert(SingleId::new(20, 0, 0, 0).unwrap(), 10);
        map.insert(SingleId::new(20, 1, 0, 0).unwrap(), 20);
        map.insert(SingleId::new(20, 2, 0, 0).unwrap(), 10);

        let json = serde_json::to_string(&map).unwrap();
        let restored: SpatialIdMap<i32> = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.count(), map.count());
        for (flex_id, value) in map.iter() {
            let (_, restored_value) = restored.get(&flex_id).next().unwrap();
            assert_eq!(restored_value, value);
        }
    }

    #[test]
    fn deserialize_rejects_out_of_range_zoom_level() {
        let json = r#"{"$schema":"https://airbee-project.github.io/schemas/json/v1.0.json","meta":{"version":"v1.0","description":""},"option":{},"data":[{"name":"","value":[10],"ids":[{"z":68,"f":[0],"x":[0],"y":[0],"ref":0}]}]}"#;
        assert!(serde_json::from_str::<SpatialIdMap<i32>>(json).is_err());
    }
}
