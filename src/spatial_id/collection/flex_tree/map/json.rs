#[cfg(feature = "serde")]
use serde::Serialize;

use crate::{FlexTreeMap, RangeId, SpatialId};

#[cfg(feature = "serde")]
#[derive(Serialize)]
struct OutputMeta {
    version: &'static str,
    description: &'static str,
}

#[cfg(feature = "serde")]
#[derive(Serialize)]
struct OutputOption {}

#[cfg(feature = "serde")]
#[derive(Serialize)]
struct OutputId {
    z: u8,
    f: Vec<i32>,
    x: Vec<u32>,
    y: Vec<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    i: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    t: Option<Vec<u64>>,
    #[serde(rename = "ref")]
    #[serde(skip_serializing_if = "Option::is_none")]
    ref_idx: Option<usize>,
}

#[cfg(feature = "serde")]
#[derive(Serialize)]
struct OutputData<'a, V> {
    name: &'static str,
    value: &'a Vec<&'a V>,
    ids: Vec<OutputId>,
}

#[cfg(feature = "serde")]
#[derive(Serialize)]
struct OutputRoot<'a, V> {
    #[serde(rename = "$schema")]
    schema: &'static str,
    meta: OutputMeta,
    option: OutputOption,
    data: Vec<OutputData<'a, V>>,
}

#[cfg(feature = "serde")]
impl<V> FlexTreeMap<V>
where
    V: PartialEq + Clone + Serialize,
{
    pub fn to_json(&self) -> String {
        // Collect unique values to build the value array
        let mut unique_values: Vec<&V> = Vec::new();
        for (_, val) in self.iter() {
            if !unique_values.contains(&val) {
                unique_values.push(val);
            }
        }

        let ids: Vec<_> = self
            .iter()
            .map(|(flex_id, val)| {
                let range_id = RangeId::from(&flex_id);
                let f = range_id.f();
                let x = range_id.x();
                let y = range_id.y();
                let temp = range_id.temporal();

                OutputId {
                    z: range_id.z(),
                    f: if f[0] == f[1] { vec![f[0]] } else { f.to_vec() },
                    x: if x[0] == x[1] { vec![x[0]] } else { x.to_vec() },
                    y: if y[0] == y[1] { vec![y[0]] } else { y.to_vec() },
                    i: if !temp.is_whole() {
                        Some(temp.i())
                    } else {
                        None
                    },
                    t: if !temp.is_whole() {
                        let t = temp.t();
                        Some(if t[0] == t[1] { vec![t[0]] } else { t.to_vec() })
                    } else {
                        None
                    },
                    ref_idx: unique_values.iter().position(|&v| v == val),
                }
            })
            .collect();

        let root = OutputRoot {
            schema: "https://airbee-project.github.io/schemas/json/v1.0.json",
            meta: OutputMeta {
                version: "v1.0",
                description: "",
            },
            option: OutputOption {},
            data: vec![OutputData {
                name: "",
                value: &unique_values,
                ids,
            }],
        };

        serde_json::to_string(&root).unwrap()
    }
}
