use alloc::vec::Vec;
use serde::ser::{Serialize, SerializeSeq, SerializeStruct, Serializer};

use crate::{RangeId, SpatialId, SpatialIdMap, SpatialIdSet, SpatialIdTable};

const SCHEMA: &str = "https://airbee-project.github.io/schemas/json/v1.0.json";

fn serialize_envelope<S: Serializer, F>(serializer: S, write_data: F) -> Result<S::Ok, S::Error>
where
    F: FnOnce(&mut <S as Serializer>::SerializeStruct) -> Result<(), S::Error>,
{
    let mut root = serializer.serialize_struct("Envelope", 4)?;
    root.serialize_field("$schema", SCHEMA)?;

    #[derive(serde::Serialize)]
    struct Meta {
        version: &'static str,
        description: &'static str,
    }
    root.serialize_field(
        "meta",
        &Meta {
            version: "v1.0",
            description: "",
        },
    )?;

    #[derive(serde::Serialize)]
    struct OptionObj {}
    root.serialize_field("option", &OptionObj {})?;

    write_data(&mut root)?;

    root.end()
}

struct CollapsedPair<T>(T, T);
impl<T: Serialize + PartialEq> Serialize for CollapsedPair<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if self.0 == self.1 {
            let mut seq = serializer.serialize_seq(Some(1))?;
            seq.serialize_element(&self.0)?;
            seq.end()
        } else {
            let mut seq = serializer.serialize_seq(Some(2))?;
            seq.serialize_element(&self.0)?;
            seq.serialize_element(&self.1)?;
            seq.end()
        }
    }
}

struct RangeIdSer(RangeId);
impl Serialize for RangeIdSer {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let f = self.0.f();
        let x = self.0.x();
        let y = self.0.y();
        let temp = self.0.temporal();

        let has_temporal = !temp.is_whole();
        let fields = if has_temporal { 6 } else { 4 };
        let mut st = serializer.serialize_struct("SpatialTemporalId", fields)?;
        st.serialize_field("z", &self.0.z())?;
        st.serialize_field("f", &CollapsedPair(f[0], f[1]))?;
        st.serialize_field("x", &CollapsedPair(x[0], x[1]))?;
        st.serialize_field("y", &CollapsedPair(y[0], y[1]))?;

        if has_temporal {
            st.serialize_field("i", &temp.i().seconds())?;
            st.serialize_field("t", &[temp.t()])?;
        }
        st.end()
    }
}

struct RefIdSer {
    range: RangeId,
    ref_idx: usize,
}
impl Serialize for RefIdSer {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let f = self.range.f();
        let x = self.range.x();
        let y = self.range.y();
        let temp = self.range.temporal();

        let has_temporal = !temp.is_whole();
        let fields = if has_temporal { 7 } else { 5 };
        let mut st = serializer.serialize_struct("RefId", fields)?;
        st.serialize_field("z", &self.range.z())?;
        st.serialize_field("f", &CollapsedPair(f[0], f[1]))?;
        st.serialize_field("x", &CollapsedPair(x[0], x[1]))?;
        st.serialize_field("y", &CollapsedPair(y[0], y[1]))?;

        if has_temporal {
            st.serialize_field("i", &temp.i().seconds())?;
            st.serialize_field("t", &[temp.t()])?;
        }
        st.serialize_field("ref", &self.ref_idx)?;
        st.end()
    }
}

impl Serialize for SpatialIdSet {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_envelope(serializer, |root| {
            #[derive(serde::Serialize)]
            struct DataObj {
                name: &'static str,
                ids: Vec<RangeIdSer>,
            }
            let ids: Vec<_> = self.iter().map(|f| RangeIdSer(RangeId::from(&f))).collect();
            let data = vec![DataObj { name: "", ids }];
            root.serialize_field("data", &data)
        })
    }
}

impl<V> Serialize for SpatialIdMap<V>
where
    V: crate::spatial_id::collection::flex_tree::ptr::SafeValue + Serialize + PartialEq,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut unique: Vec<&V> = Vec::new();
        for (_, val) in self.iter() {
            if !unique.contains(&val) {
                unique.push(val);
            }
        }

        serialize_envelope(serializer, |root| {
            #[derive(serde::Serialize)]
            struct DataObj<'a, V: Serialize> {
                name: &'static str,
                value: Vec<&'a V>,
                ids: Vec<RefIdSer>,
            }
            let ids: Vec<_> = self
                .iter()
                .map(|(flex_id, val)| RefIdSer {
                    range: RangeId::from(&flex_id),
                    ref_idx: unique.iter().position(|&u| u == val).unwrap(),
                })
                .collect();
            let data = vec![DataObj {
                name: "",
                value: unique,
                ids,
            }];
            root.serialize_field("data", &data)
        })
    }
}

impl<V> Serialize for SpatialIdTable<V>
where
    V: crate::spatial_id::collection::flex_tree::ptr::SafeValue + Ord + Serialize,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut unique: Vec<&V> = Vec::new();
        for (_, val) in self.iter() {
            if !unique.contains(&val) {
                unique.push(val);
            }
        }

        serialize_envelope(serializer, |root| {
            #[derive(serde::Serialize)]
            struct DataObj<'a, V: Serialize> {
                name: &'static str,
                value: Vec<&'a V>,
                ids: Vec<RefIdSer>,
            }
            let ids: Vec<_> = self
                .iter()
                .map(|(flex_id, val)| RefIdSer {
                    range: RangeId::from(&flex_id),
                    ref_idx: unique.iter().position(|&u| u == val).unwrap(),
                })
                .collect();
            let data = vec![DataObj {
                name: "",
                value: unique,
                ids,
            }];
            root.serialize_field("data", &data)
        })
    }
}
