use alloc::string::String;
use alloc::vec::Vec;

use crate::{CellValue, JsonValue, RangeId, SpatialIdTable};

use crate::spatial_id::collection::json::{write_envelope_open, write_id_open};

impl<V> SpatialIdTable<V>
where
    V: CellValue + JsonValue,
{
    /// このテーブルを <https://airbee-project.github.io/schemas/json/v1.0.json> 準拠の
    /// JSON 文字列へ書き出す。
    pub fn to_json(&self) -> String {
        // 値を重複なく集め、各値に配列内インデックスを割り当てる。
        let mut values: Vec<&V> = Vec::new();
        let mut cells: Vec<(RangeId, usize)> = Vec::new();

        for (sid, value) in self.iter() {
            let idx = match values.iter().position(|v| *v == value) {
                Some(i) => i,
                None => {
                    values.push(value);
                    values.len() - 1
                }
            };
            cells.push((RangeId::from(&sid), idx));
        }

        let mut out = String::new();
        write_envelope_open(&mut out);

        out.push_str("\"value\":[");
        for (i, value) in values.iter().enumerate() {
            if i != 0 {
                out.push(',');
            }
            value.write_json(&mut out);
        }
        out.push_str("],\"ids\":[");

        for (i, (range_id, idx)) in cells.iter().enumerate() {
            if i != 0 {
                out.push(',');
            }
            write_id_open(&mut out, range_id);
            out.push_str(",\"ref\":");
            idx.write_json(&mut out);
            out.push('}');
        }

        out.push_str("]}]}");
        out
    }
}
