use alloc::string::String;
use alloc::vec::Vec;

use crate::{JsonValue, RangeId, SpatialIdMap};

use super::super::json::{write_envelope_open, write_id_open};

impl<V> SpatialIdMap<V>
where
    V: crate::spatial_id::collection::tree::ptr::SafeValue + JsonValue,
{
    /// このマップを <https://airbee-project.github.io/schemas/json/v1.0.json> 準拠の JSON 文字列へ
    /// 書き出す。外部クレート（serde 等）に依存せず、値型 `V` が [`JsonValue`] を実装していれば
    /// いつでも利用できる。
    ///
    /// 値は `data[].value` に重複なく列挙し、各空間 ID は `ref` でその添字を参照する。
    ///
    /// # 例
    ///
    /// ```
    /// use kasane_logic::{SingleId, SpatialIdMap};
    /// let mut map: SpatialIdMap<i32> = SpatialIdMap::new();
    /// map.insert(SingleId::new(20, 0, 0, 0).unwrap(), 10);
    ///
    /// let json = map.to_json();
    /// assert!(json.contains("\"value\":[10]"));
    /// assert!(json.contains("\"ref\":0"));
    /// ```
    pub fn to_json(&self) -> String {
        // 値を出現順で重複排除し、各 ID から添字（ref）で参照する。
        let mut unique: Vec<&V> = Vec::new();
        for (_, val) in self.iter() {
            if !unique.contains(&val) {
                unique.push(val);
            }
        }

        let mut out = String::new();
        write_envelope_open(&mut out);

        out.push_str("\"value\":[");
        let mut first = true;
        for v in &unique {
            if !first {
                out.push(',');
            }
            first = false;
            (**v).write_json(&mut out);
        }
        out.push_str("],\"ids\":[");

        let mut first = true;
        for (flex_id, val) in self.iter() {
            if !first {
                out.push(',');
            }
            first = false;

            let range_id = RangeId::from(&flex_id);
            write_id_open(&mut out, &range_id);
            if let Some(idx) = unique.iter().position(|&u| u == val) {
                out.push_str(",\"ref\":");
                idx.write_json(&mut out);
            }
            out.push('}');
        }

        out.push_str("]}]}");
        out
    }
}

#[cfg(test)]
mod tests {
    use crate::{SingleId, SpatialIdMap};
    use alloc::string::ToString;

    #[test]
    fn to_json_lists_unique_values_and_refs() {
        let mut map = SpatialIdMap::<i32>::new();
        map.insert(SingleId::new(20, 0, 0, 0).unwrap(), 10);
        map.insert(SingleId::new(20, 1, 0, 0).unwrap(), 20);
        map.insert(SingleId::new(20, 2, 0, 0).unwrap(), 10); // 値 10 を再利用

        let json = map.to_json();

        // 値は数値としてそのまま（クォートされない）、重複は1つに集約。
        assert!(json.contains("\"value\":[10,20]"));
        assert!(json.contains("\"ref\":0"));
        assert!(json.contains("\"ref\":1"));
        assert!(json.contains("\"z\":20"));
    }

    #[test]
    fn to_json_quotes_and_escapes_string_values() {
        let mut map = SpatialIdMap::<String>::new();
        map.insert(SingleId::new(20, 0, 0, 0).unwrap(), "a\"b".to_string());

        let json = map.to_json();
        // 文字列値はクォートされ、" がエスケープされる。
        assert!(json.contains("\"value\":[\"a\\\"b\"]"));
    }

    #[test]
    fn to_json_empty_map_has_empty_value_and_ids() {
        let map = SpatialIdMap::<i32>::new();
        let json = map.to_json();
        assert!(json.contains("\"value\":[]"));
        assert!(json.contains("\"ids\":[]"));
    }
}
