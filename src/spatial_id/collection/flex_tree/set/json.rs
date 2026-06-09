#[allow(unused_imports)]
use alloc::vec::Vec;
#[allow(unused_imports)]
use alloc::string::{String, ToString};
#[allow(unused_imports)]
use alloc::boxed::Box;
#[allow(unused_imports)]
use alloc::rc::Rc;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::{RangeId, SpatialId, SpatialIdSet};

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
    t: Option<u64>,
}

#[cfg(feature = "serde")]
#[derive(Serialize)]
struct OutputData {
    name: &'static str,
    ids: Vec<OutputId>,
}

#[cfg(feature = "serde")]
#[derive(Serialize)]
struct OutputRoot {
    #[serde(rename = "$schema")]
    schema: &'static str,
    meta: OutputMeta,
    option: OutputOption,
    data: Vec<OutputData>,
}

#[cfg(feature = "serde")]
impl SpatialIdSet {
    pub fn to_json(&self) -> String {
        let ids: Vec<_> = self
            .iter()
            .map(|flex_id| {
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
                        Some(temp.t())
                    } else {
                        None
                    },
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
            data: vec![OutputData { name: "", ids }],
        };

        serde_json::to_string(&root).unwrap()
    }
}
