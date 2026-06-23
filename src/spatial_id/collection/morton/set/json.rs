use alloc::string::String;

use crate::{RangeId, SpatialIdSet};

use crate::spatial_id::collection::json::{write_envelope_open, write_id_open};

impl SpatialIdSet {
    /// この集合を <https://airbee-project.github.io/schemas/json/v1.0.json> 準拠の JSON 文字列へ
    /// 書き出す。
    pub fn to_json(&self) -> String {
        let mut out = String::new();
        write_envelope_open(&mut out);
        out.push_str("\"ids\":[");

        let mut first = true;
        for single in self.iter() {
            if !first {
                out.push(',');
            }
            first = false;

            let range_id = RangeId::from(&single);
            write_id_open(&mut out, &range_id);
            out.push('}');
        }

        out.push_str("]}]}");
        out
    }
}
