use alloc::string::String;

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::SpatialIdTable;

use super::super::json::{JsonError, from_json_with_values, to_json_with_values};

impl<V> SpatialIdTable<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue
        + Ord
        + Serialize
        + DeserializeOwned,
{
    /// このテーブルを <https://airbee-project.github.io/schemas/json/v1.0.json> 準拠の JSON 文字列へ
    /// 書き出す。値型 `V` が [`serde::Serialize`] を実装していればいつでも利用できる。
    ///
    /// 値は `data[].value` に重複なく列挙し、各空間 ID は `ref` でその添字を参照する。
    ///
    /// # 例
    ///
    /// ```
    /// use kasane_logic::{SingleId, SpatialIdTable};
    /// let mut table: SpatialIdTable<i32> = SpatialIdTable::new();
    /// table.insert(SingleId::new(20, 0, 0, 0).unwrap(), 10);
    ///
    /// let json = table.to_json();
    /// assert!(json.contains("\"value\":[10]"));
    /// assert!(json.contains("\"ref\":0"));
    /// ```
    pub fn to_json(&self) -> String {
        to_json_with_values(self.iter())
    }

    /// [`to_json`](Self::to_json) が書き出した JSON 文字列からテーブルを復元する。
    /// 値型 `V` が [`serde::de::DeserializeOwned`] を実装していればいつでも利用できる。
    ///
    /// # 例
    ///
    /// ```
    /// use kasane_logic::{SingleId, SpatialIdTable};
    /// let mut table: SpatialIdTable<i32> = SpatialIdTable::new();
    /// table.insert(SingleId::new(20, 0, 0, 0).unwrap(), 10);
    ///
    /// let restored = SpatialIdTable::<i32>::from_json(&table.to_json()).unwrap();
    /// assert_eq!(restored.count(), table.count());
    /// ```
    pub fn from_json(json: &str) -> Result<Self, JsonError> {
        let mut table = Self::new();
        for (range_id, value) in from_json_with_values(json)? {
            table.insert(range_id, value);
        }
        Ok(table)
    }
}

#[cfg(test)]
mod tests {
    use crate::{SingleId, SpatialIdTable};
    use alloc::string::ToString;

    #[test]
    fn to_json_lists_unique_values_and_refs() {
        let mut table = SpatialIdTable::<i32>::new();
        table.insert(SingleId::new(20, 0, 0, 0).unwrap(), 10);
        table.insert(SingleId::new(20, 1, 0, 0).unwrap(), 20);
        table.insert(SingleId::new(20, 2, 0, 0).unwrap(), 10); // 値 10 を再利用

        let json = table.to_json();

        // 値は数値としてそのまま（クォートされない）、重複は1つに集約。
        assert!(json.contains("\"value\":[10,20]"));
        assert!(json.contains("\"ref\":0"));
        assert!(json.contains("\"ref\":1"));
        assert!(json.contains("\"z\":20"));
    }

    #[test]
    fn to_json_quotes_and_escapes_string_values() {
        let mut table = SpatialIdTable::<String>::new();
        table.insert(SingleId::new(20, 0, 0, 0).unwrap(), "a\"b".to_string());

        let json = table.to_json();
        // 文字列値はクォートされ、" がエスケープされる。
        assert!(json.contains("\"value\":[\"a\\\"b\"]"));
    }

    #[test]
    fn to_json_empty_table_has_empty_value_and_ids() {
        let table = SpatialIdTable::<i32>::new();
        let json = table.to_json();
        assert!(json.contains("\"value\":[]"));
        assert!(json.contains("\"ids\":[]"));
    }

    #[test]
    fn from_json_round_trips_values_and_ids() {
        let mut table = SpatialIdTable::<i32>::new();
        table.insert(SingleId::new(20, 0, 0, 0).unwrap(), 10);
        table.insert(SingleId::new(20, 1, 0, 0).unwrap(), 20);
        table.insert(SingleId::new(20, 2, 0, 0).unwrap(), 10);

        let json = table.to_json();
        let restored = SpatialIdTable::<i32>::from_json(&json).unwrap();

        assert_eq!(restored.count(), table.count());
        for (flex_id, value) in table.iter() {
            let (_, restored_value) = restored.get(&flex_id).next().unwrap();
            assert_eq!(restored_value, value);
        }
    }

    #[test]
    fn from_json_rejects_out_of_range_zoom_level() {
        let json = r#"{"$schema":"https://airbee-project.github.io/schemas/json/v1.0.json","meta":{"version":"v1.0","description":""},"option":{},"data":[{"name":"","value":[10],"ids":[{"z":68,"f":[0],"x":[0],"y":[0],"ref":0}]}]}"#;
        assert!(SpatialIdTable::<i32>::from_json(json).is_err());
    }
}
