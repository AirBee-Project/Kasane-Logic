//! コレクションの JSON 書き出しで共有する、外部クレート非依存の軽量ユーティリティ。
//!
//! serde などに頼らず `alloc::string::String` への手書きで
//! <https://airbee-project.github.io/schemas/json/v1.0.json> 準拠の JSON を組み立てる。

use alloc::string::{String, ToString};

use crate::{RangeId, SpatialId};

/// JSON のリテラル片として直列化できる値。
///
/// [`SpatialIdTable::to_json`](crate::SpatialIdTable::to_json) が値型 `V` を JSON へ書き出す際に
/// 用いる。標準の整数・浮動小数・真偽・文字列には実装済み。独自の値型を JSON 出力したい場合は
/// このトレイトを実装する。スキーマの `value` 配列は「文字列 / 数値 / 真偽 / オブジェクト」の
/// いずれかを要素にとるため、実装はそのいずれかの JSON 片を書き出すこと。
pub trait JsonValue {
    /// 自身を JSON のリテラル片（例: `42`、`"foo"`、`true`）として `out` へ追記する。
    fn write_json(&self, out: &mut String);
}

macro_rules! impl_json_value_int {
    ($($t:ty),*) => {$(
        impl JsonValue for $t {
            fn write_json(&self, out: &mut String) {
                out.push_str(&self.to_string());
            }
        }
    )*};
}
impl_json_value_int!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);

macro_rules! impl_json_value_float {
    ($($t:ty),*) => {$(
        impl JsonValue for $t {
            fn write_json(&self, out: &mut String) {
                if self.is_finite() {
                    out.push_str(&self.to_string());
                } else {
                    // JSON は NaN/Infinity を表現できないため null とする。
                    out.push_str("null");
                }
            }
        }
    )*};
}
impl_json_value_float!(f32, f64);

impl JsonValue for bool {
    fn write_json(&self, out: &mut String) {
        out.push_str(if *self { "true" } else { "false" });
    }
}

impl JsonValue for str {
    fn write_json(&self, out: &mut String) {
        write_json_string(out, self);
    }
}

impl JsonValue for String {
    fn write_json(&self, out: &mut String) {
        write_json_string(out, self);
    }
}

impl JsonValue for char {
    fn write_json(&self, out: &mut String) {
        let mut buf = [0u8; 4];
        write_json_string(out, self.encode_utf8(&mut buf));
    }
}

impl<T: JsonValue + ?Sized> JsonValue for &T {
    fn write_json(&self, out: &mut String) {
        (**self).write_json(out);
    }
}

impl JsonValue for crate::Interval {
    fn write_json(&self, out: &mut String) {
        self.seconds().write_json(out);
    }
}

impl<T: JsonValue> JsonValue for Option<T> {
    fn write_json(&self, out: &mut String) {
        match self {
            Some(v) => v.write_json(out),
            None => out.push_str("null"),
        }
    }
}

/// 文字列を JSON 文字列リテラル（前後の `"` とエスケープ込み）として書き出す。
pub(crate) fn write_json_string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                out.push_str("\\u00");
                let code = c as u32;
                out.push(char::from_digit((code >> 4) & 0xf, 16).unwrap());
                out.push(char::from_digit(code & 0xf, 16).unwrap());
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// `lo == hi` なら `[lo]`、異なれば `[lo,hi]` を書き出す（スキーマの `f`/`x`/`y` 用）。
fn write_collapsed_pair<T: JsonValue + PartialEq>(out: &mut String, lo: T, hi: T) {
    out.push('[');
    lo.write_json(out);
    if lo != hi {
        out.push(',');
        hi.write_json(out);
    }
    out.push(']');
}

/// 1つの空間IDを、スキーマの `spatialTemporalId`（`{"z":..,"f":..,"x":..,"y":..[,"i":..,"t":..]`）
/// として書き出す。閉じ波括弧 `}` は呼び出し側が付ける（Table が `"ref"` を足せるようにするため）。
///
/// `i` はスカラ整数、`t` は配列（いずれもスキーマ v1.0 準拠）。時間が全範囲（`is_whole`）の
/// ときは `i`/`t` を出力しない。
pub(crate) fn write_id_open(out: &mut String, range_id: &RangeId) {
    let f = range_id.f();
    let x = range_id.x();
    let y = range_id.y();

    out.push_str("{\"z\":");
    range_id.z().write_json(out);
    out.push_str(",\"f\":");
    write_collapsed_pair(out, f[0], f[1]);
    out.push_str(",\"x\":");
    write_collapsed_pair(out, x[0], x[1]);
    out.push_str(",\"y\":");
    write_collapsed_pair(out, y[0], y[1]);

    let temp = range_id.temporal();
    if !temp.is_whole() {
        out.push_str(",\"i\":");
        temp.i().write_json(out);
        out.push_str(",\"t\":[");
        temp.t().write_json(out);
        out.push(']');
    }
}

#[cfg(all(test, feature = "temporal_id"))]
mod tests {
    use super::write_id_open;
    use crate::{RangeId, TemporalId};
    use alloc::string::String;

    /// スキーマ v1.0 では `i` はスカラ整数、`t` は配列。シリアライザがそれに従うことを、
    /// 時間ID付きの [`RangeId`] を直接渡して検証する。
    #[test]
    fn write_id_open_emits_i_scalar_and_t_array() {
        let temporal = TemporalId::from_seconds(3600, 5).unwrap();
        let range = RangeId::new(20, [0, 0], [0, 0], [0, 0])
            .unwrap()
            .with_temporal(temporal);

        let mut out = String::new();
        write_id_open(&mut out, &range);

        assert!(out.contains("\"i\":3600"), "got: {out}");
        assert!(out.contains("\"t\":[5]"), "got: {out}");
        assert!(out.ends_with("\"t\":[5]"), "got: {out}");
    }
}

/// すべてのコレクションで共通の JSON 先頭部
/// （`{"$schema":..,"meta":..,"option":..,"data":[{"name":"",`）を書き出す。
/// 呼び出し側はこの後に `"value":[..],`（任意）と `"ids":[..]}]}` を続ける。
pub(crate) fn write_envelope_open(out: &mut String) {
    out.push_str("{\"$schema\":\"https://airbee-project.github.io/schemas/json/v1.0.json\"");
    out.push_str(",\"meta\":{\"version\":\"v1.0\",\"description\":\"\"}");
    out.push_str(",\"option\":{}");
    out.push_str(",\"data\":[{\"name\":\"\",");
}
