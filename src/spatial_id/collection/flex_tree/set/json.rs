use alloc::string::String;

use crate::SpatialIdSet;

use super::super::json::{JsonError, from_json_without_values, to_json_without_values};

impl SpatialIdSet {
    /// この集合を <https://airbee-project.github.io/schemas/json/v1.0.json> 準拠の JSON 文字列として書き出す。
    pub fn to_json(&self) -> String {
        to_json_without_values(self.iter())
    }

    /// [`to_json`](Self::to_json) が書き出した JSON 文字列から集合を復元する。
    pub fn from_json(json: &str) -> Result<Self, JsonError> {
        let mut set = Self::new();
        for range_id in from_json_without_values(json)? {
            set.insert(range_id);
        }
        Ok(set)
    }
}

#[cfg(test)]
mod tests {
    use crate::{SingleId, SpatialIdSet};

    #[test]
    fn to_json_has_schema_envelope_and_ids() {
        let mut set = SpatialIdSet::new();
        set.insert(SingleId::new(20, 0, 0, 0).unwrap());

        let json = set.to_json();

        assert!(json.starts_with(
            "{\"$schema\":\"https://airbee-project.github.io/schemas/json/v1.0.json\""
        ));
        assert!(json.contains("\"meta\":{\"version\":\"v1.0\",\"description\":\"\"}"));
        assert!(json.contains("\"option\":{}"));
        assert!(json.contains("\"data\":[{\"name\":\"\",\"ids\":["));
        assert!(json.contains("\"z\":20"));
        assert!(json.contains("\"f\":[0]"));
        // 値を持たない集合は value を出さない。
        assert!(!json.contains("\"value\""));
        // 非 temporal では i / t を出さない。
        assert!(!json.contains("\"i\":"));
        assert!(!json.contains("\"t\":"));
    }

    #[test]
    fn to_json_empty_set_has_empty_ids() {
        let set = SpatialIdSet::new();
        assert!(set.to_json().contains("\"ids\":[]"));
    }

    #[test]
    fn to_json_collapses_and_expands_ranges() {
        use crate::RangeId;
        let mut set = SpatialIdSet::new();
        // x は範囲 [0,1] を持つ（z20）。f/y は単一。
        set.insert(RangeId::new(20, [0, 0], [0, 1], [0, 0]).unwrap());

        let json = set.to_json();
        assert!(json.contains("\"x\":[0,1]"));
        assert!(json.contains("\"f\":[0]"));
    }

    #[test]
    fn from_json_round_trips_a_set() {
        let mut set = SpatialIdSet::new();
        set.insert(SingleId::new(20, 0, 0, 0).unwrap());
        set.insert(SingleId::new(20, 1, 0, 0).unwrap());

        let json = set.to_json();
        let restored = SpatialIdSet::from_json(&json).unwrap();

        assert_eq!(restored.count(), set.count());
        for flex_id in set.iter() {
            assert!(restored.get(&flex_id).next().is_some());
        }
    }

    #[test]
    fn from_json_rejects_malformed_pair_length() {
        let json = r#"{"$schema":"https://airbee-project.github.io/schemas/json/v1.0.json","meta":{"version":"v1.0","description":""},"option":{},"data":[{"name":"","ids":[{"z":20,"f":[0,1,2],"x":[0],"y":[0]}]}]}"#;
        assert!(SpatialIdSet::from_json(json).is_err());
    }
}
