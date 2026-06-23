use alloc::string::String;

use crate::{RangeId, SpatialIdSet};

use crate::spatial_id::collection::json::{write_envelope_open, write_id_open};

impl SpatialIdSet {
    /// この集合を <https://airbee-project.github.io/schemas/json/v1.0.json> 準拠の JSON 文字列へ
    /// 書き出す。外部クレート（serde 等）に依存せず、いつでも利用できる。
    ///
    /// 値を持たない集合なので `data[].value` は出力せず、占有空間を `ids` として並べる。
    ///
    /// # 例
    ///
    /// ```
    /// use kasane_logic::{SingleId, SpatialIdSet};
    /// let mut set = SpatialIdSet::new();
    /// set.insert(SingleId::new(20, 0, 0, 0).unwrap());
    ///
    /// let json = set.to_json();
    /// assert!(json.contains("\"z\":20"));
    /// assert!(json.contains("\"ids\":["));
    /// ```
    pub fn to_json(&self) -> String {
        let mut out = String::new();
        write_envelope_open(&mut out);
        out.push_str("\"ids\":[");

        let mut first = true;
        for flex_id in self.iter() {
            if !first {
                out.push(',');
            }
            first = false;

            let range_id = RangeId::from(&flex_id);
            write_id_open(&mut out, &range_id);
            out.push('}');
        }

        out.push_str("]}]}");
        out
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
}
