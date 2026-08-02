//! 時間軸まわりの実装。
//!
//! 空間 ID の3軸（F / X / Y）に対して、[Ouranos 4D 時空間ID仕様](https://www.ipa.go.jp/)の
//! Temporal ID `{i}/{t}` を扱うための型と変換をここへ集約している。
//!
//! | モジュール | 役割 | 公開範囲 |
//! |---|---|---|
//! | [`interval`](crate::spatial_id::time::interval) | 時間間隔 `{i}`（秒数）。**公開 API に出てくる唯一の時間の型** | 公開 |
//! | [`interval_set`](crate::spatial_id::time::interval_set) | 読み出し時に `{i}` として使ってよい候補集合 | 公開 |
//! | `cells` | 絶対秒区間と FlexTree の2進セルの相互変換 | クレート内部 |
//!
//! 「いつか」を表す `{t}` は専用の型を持たず、各 ID（[`SingleId`](crate::SingleId) /
//! [`RangeId`](crate::RangeId) / [`FlexId`](crate::FlexId)）がフィールドとして直接持つ。
//!
//! # ここに無いもの
//!
//! 時間軸のズームレベル [`TZoomLevel`](crate::TZoomLevel) は
//! [`zoom_level`](crate::spatial_id::zoom_level) 側にある。空間軸の
//! [`ZoomLevel`](crate::ZoomLevel) と `Zoom<LIMIT>` という同じ土台を共有しており、
//! 生成・深化・範囲チェックの式が共通だからである。
//!
//! 木から読み出したセルを時間方向へ結合する層
//! （`collection::flex_tree::coalesce`）も、コレクション固有の処理なのでそちらに置いている。

/// 時間間隔 `{i}`。公開 API に出てくる唯一の時間の型。
pub mod interval;
/// 読み出し時に `{i}` として使ってよい時間間隔の候補集合。
pub mod interval_set;

/// 絶対秒区間と FlexTree の2進セルの相互変換（クレート内部専用）。
pub(crate) mod cells;

pub use interval::Interval;
pub use interval_set::IntervalSet;
