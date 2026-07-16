//! コレクションの JSON 変換で共有する、serde ベースのユーティリティ。
//!
//! <https://airbee-project.github.io/schemas/json/v1.0.json> 準拠の JSON を
//! `serde`/`serde_json`（いずれも `alloc` feature のみで動作し、no_std 環境でも使える）を使って
//! 組み立て・復元する。値型 `V` は [`serde::Serialize`]/[`serde::de::DeserializeOwned`] を実装して
//! いれば任意の型を使える。
//!
//! スキーマの `f`/`x`/`y` の `[lo]`/`[lo,hi]` 省略や、`i`/`t` の条件付き省略（全時間のときは
//! 出さない）は `#[derive(Serialize, Deserialize)]` だけでは表現できないため、`IdEntry` だけは
//! `Serializer`/`Deserializer` を直接叩く手書き実装にしている。

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{FlexId, RangeId, SpatialId};

const SCHEMA_URL: &str = "https://airbee-project.github.io/schemas/json/v1.0.json";

/// 1つの空間IDを、スキーマの `spatialTemporalId` として書き出す／読み込む。
///
/// `ref` は値ありコレクション（Table/Map）だけが使う、`data[].value` への添字。
struct IdEntry {
    range_id: RangeId,
    r#ref: Option<usize>,
}

fn serialize_pair<M, T>(map: &mut M, key: &'static str, pair: [T; 2]) -> Result<(), M::Error>
where
    M: SerializeMap,
    T: Serialize + PartialEq + Copy,
{
    if pair[0] == pair[1] {
        map.serialize_entry(key, &[pair[0]])
    } else {
        map.serialize_entry(key, &pair)
    }
}

fn deserialize_pair<T: Copy>(values: Vec<T>) -> Result<[T; 2], &'static str> {
    match values.as_slice() {
        [v] => Ok([*v, *v]),
        [a, b] => Ok([*a, *b]),
        _ => Err("expected an array of length 1 or 2"),
    }
}

/// `z`/`f`/`x`/`y` と、あれば `i`/`t` から [`RangeId`] を組み立てる。
///
/// `temporal_id` feature が無効なときは常に全時間（`WHOLE`）として扱う
/// （[`TemporalId`](crate::TemporalId) 無効時スタブと同じ振る舞い）。
fn build_range_id(
    z: u8,
    f: [i32; 2],
    x: [u32; 2],
    y: [u32; 2],
    temporal_pair: Option<(u64, u64)>,
) -> Result<RangeId, crate::Error> {
    match temporal_pair {
        #[cfg(feature = "temporal_id")]
        Some((i, t)) => {
            let temporal = crate::TemporalId::new(i, t)?;
            RangeId::new_with_temporal(z, f, x, y, temporal)
        }
        #[cfg(not(feature = "temporal_id"))]
        Some(_) => RangeId::new(z, f, x, y),
        None => RangeId::new(z, f, x, y),
    }
}

impl Serialize for IdEntry {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let temporal = self.range_id.temporal();

        let mut len = 4;
        if !temporal.is_whole() {
            len += 2;
        }
        if self.r#ref.is_some() {
            len += 1;
        }

        let mut map = serializer.serialize_map(Some(len))?;
        map.serialize_entry("z", &self.range_id.z())?;
        serialize_pair(&mut map, "f", self.range_id.f())?;
        serialize_pair(&mut map, "x", self.range_id.x())?;
        serialize_pair(&mut map, "y", self.range_id.y())?;
        if !temporal.is_whole() {
            map.serialize_entry("i", &temporal.i())?;
            map.serialize_entry("t", &[temporal.t()])?;
        }
        if let Some(r) = self.r#ref {
            map.serialize_entry("ref", &r)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for IdEntry {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct IdEntryVisitor;

        impl<'de> Visitor<'de> for IdEntryVisitor {
            type Value = IdEntry;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a spatialTemporalId object")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut z: Option<u8> = None;
                let mut f: Option<[i32; 2]> = None;
                let mut x: Option<[u32; 2]> = None;
                let mut y: Option<[u32; 2]> = None;
                let mut i: Option<u64> = None;
                let mut t: Option<u64> = None;
                let mut r#ref: Option<usize> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "z" => z = Some(map.next_value()?),
                        "f" => {
                            f = Some(
                                deserialize_pair(map.next_value()?).map_err(de::Error::custom)?,
                            )
                        }
                        "x" => {
                            x = Some(
                                deserialize_pair(map.next_value()?).map_err(de::Error::custom)?,
                            )
                        }
                        "y" => {
                            y = Some(
                                deserialize_pair(map.next_value()?).map_err(de::Error::custom)?,
                            )
                        }
                        "i" => i = Some(map.next_value()?),
                        "t" => {
                            let pair =
                                deserialize_pair(map.next_value()?).map_err(de::Error::custom)?;
                            if pair[0] != pair[1] {
                                panic!(
                                    "range of temporal id is not currently supported in the reader"
                                );
                            }
                            t = Some(pair[0]);
                        }
                        "ref" => r#ref = Some(map.next_value()?),
                        _ => {
                            let _ = map.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }

                let z = z.ok_or_else(|| de::Error::missing_field("z"))?;
                let z_level =
                    crate::spatial_id::zoom_level::ZoomLevel::new(z).map_err(de::Error::custom)?;
                let f = f.unwrap_or([z_level.f_min(), z_level.f_max()]);
                let x = x.unwrap_or([0, z_level.xy_max()]);
                let y = y.unwrap_or([0, z_level.xy_max()]);
                let temporal_pair = match (i, t) {
                    (Some(i), Some(t)) => Some((i, t)),
                    (None, None) => None,
                    _ => {
                        return Err(de::Error::custom(
                            "\"i\" and \"t\" must both be present or both be absent",
                        ));
                    }
                };

                let range_id =
                    build_range_id(z, f, x, y, temporal_pair).map_err(de::Error::custom)?;

                Ok(IdEntry { range_id, r#ref })
            }
        }

        deserializer.deserialize_map(IdEntryVisitor)
    }
}

#[derive(Serialize, Deserialize)]
struct Meta {
    version: String,
    description: String,
}

impl Meta {
    fn v1() -> Self {
        Meta {
            version: "v1.0".to_string(),
            description: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Options {}

#[derive(Serialize)]
struct EnvelopeOut<D: Serialize> {
    #[serde(rename = "$schema")]
    schema: &'static str,
    meta: Meta,
    option: Options,
    data: [D; 1],
}

#[derive(Deserialize)]
struct EnvelopeIn<D> {
    #[serde(rename = "$schema")]
    #[allow(dead_code)]
    schema: String,
    #[allow(dead_code)]
    meta: Meta,
    #[allow(dead_code)]
    option: Options,
    data: Vec<D>,
}

fn take_single_entry<D>(envelope: EnvelopeIn<D>) -> Result<D, String> {
    let mut data = envelope.data;
    if data.len() != 1 {
        return Err(format!(
            "expected \"data\" to contain exactly 1 entries, found {}",
            data.len()
        ));
    }
    Ok(data.remove(0))
}

#[derive(Serialize)]
struct ValuedDataEntryOut<'a, V: Serialize> {
    name: &'static str,
    value: Vec<&'a V>,
    ids: Vec<IdEntry>,
}

#[derive(Deserialize)]
struct ValuedDataEntryIn<V> {
    value: Vec<V>,
    ids: Vec<IdEntry>,
}

#[derive(Serialize, Deserialize)]
struct PlainDataEntry {
    name: String,
    ids: Vec<IdEntry>,
}

/// 値ありコレクション（Table/Map）向けの JSON 書き出し。
///
/// 値は出現順で重複排除して `value` に列挙し、各空間 ID は `ref` でその添字を参照する。
pub(crate) fn serialize_with_values<'a, V, S>(
    iter: impl Iterator<Item = (FlexId, &'a V)>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    V: Serialize + PartialEq + 'a,
    S: Serializer,
{
    let mut unique: Vec<&'a V> = Vec::new();
    let mut ids: Vec<IdEntry> = Vec::new();

    for (flex_id, val) in iter {
        let idx = match unique.iter().position(|&u| u == val) {
            Some(idx) => idx,
            None => {
                unique.push(val);
                unique.len() - 1
            }
        };
        ids.push(IdEntry {
            range_id: RangeId::from(&flex_id),
            r#ref: Some(idx),
        });
    }

    let envelope = EnvelopeOut {
        schema: SCHEMA_URL,
        meta: Meta::v1(),
        option: Options {},
        data: [ValuedDataEntryOut {
            name: "",
            value: unique,
            ids,
        }],
    };

    envelope.serialize(serializer)
}

/// 値なしコレクション（Set）向けの JSON 書き出し。
pub(crate) fn serialize_without_values<S>(
    iter: impl Iterator<Item = FlexId>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let ids: Vec<IdEntry> = iter
        .map(|flex_id| IdEntry {
            range_id: RangeId::from(&flex_id),
            r#ref: None,
        })
        .collect();

    let envelope = EnvelopeOut {
        schema: SCHEMA_URL,
        meta: Meta::v1(),
        option: Options {},
        data: [PlainDataEntry {
            name: String::new(),
            ids,
        }],
    };

    envelope.serialize(serializer)
}

/// 値ありコレクション（Table/Map）向けの JSON 復元。
///
/// `data[].value` と各 `ids[].ref` から `(RangeId, V)` の列を組み立てる。
pub(crate) fn deserialize_with_values<'de, V, D>(
    deserializer: D,
) -> Result<Vec<(RangeId, V)>, D::Error>
where
    V: Deserialize<'de> + Clone,
    D: Deserializer<'de>,
{
    let envelope: EnvelopeIn<ValuedDataEntryIn<V>> = EnvelopeIn::deserialize(deserializer)?;
    let entry = take_single_entry(envelope).map_err(de::Error::custom)?;
    let values = entry.value;

    let mut out = Vec::with_capacity(entry.ids.len());
    for id in entry.ids {
        let value = match id.r#ref {
            Some(idx) => values
                .get(idx)
                .cloned()
                .ok_or_else(|| de::Error::custom(format!("\"ref\" index {idx} out of range")))?,
            None => {
                return Err(de::Error::custom("id entry is missing \"ref\""));
            }
        };
        out.push((id.range_id, value));
    }
    Ok(out)
}

/// 値なしコレクション（Set）向けの JSON 復元。
pub(crate) fn deserialize_without_values<'de, D>(deserializer: D) -> Result<Vec<RangeId>, D::Error>
where
    D: Deserializer<'de>,
{
    let envelope: EnvelopeIn<PlainDataEntry> = EnvelopeIn::deserialize(deserializer)?;
    let entry = take_single_entry(envelope).map_err(de::Error::custom)?;
    Ok(entry.ids.into_iter().map(|id| id.range_id).collect())
}

#[cfg(test)]
mod tests {
    use alloc::format;

    // `SpatialIdSet`/`SpatialIdTable`/`SpatialIdMap` の木は f/x/y の空間分割のみを保持し、
    // 挿入した ID の temporal 成分は伝播しない（本リファクタ以前からの既存挙動）。そのため
    // `i`/`t` の直列化は木を経由せず [`super::IdEntry`] を直接使って検証する。
    #[cfg(feature = "temporal_id")]
    #[test]
    fn round_trips_temporal_i_scalar_and_t_array() {
        use super::IdEntry;
        use crate::{RangeId, TemporalId};

        let temporal = TemporalId::new(3600, 5).unwrap();
        let range_id = RangeId::new_with_temporal(20, [0, 0], [0, 0], [0, 0], temporal).unwrap();
        let entry = IdEntry {
            range_id: range_id.clone(),
            r#ref: None,
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"i\":3600"));
        assert!(json.contains("\"t\":[5]"));

        let restored: IdEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.range_id, range_id);
    }

    #[test]
    fn rejects_invalid_data_count() {
        let json = format!(
            "{{\"$schema\":\"{}\",\"meta\":{{\"version\":\"v1.0\",\"description\":\"\"}},\"option\":{{}},\"data\":[]}}",
            super::SCHEMA_URL
        );
        let mut deserializer = serde_json::Deserializer::from_str(&json);
        let err = super::deserialize_without_values(&mut deserializer).unwrap_err();
        assert!(
            err.to_string()
                .contains("expected \"data\" to contain exactly 1 entries, found 0")
        );
    }
}
