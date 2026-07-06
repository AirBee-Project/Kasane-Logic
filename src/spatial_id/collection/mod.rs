//! 時空間IDコレクション。
//!
//! # レイヤ構造
//!
//! モジュールは下から上への積み重ねで、**ディレクトリ順 = 依存順**である。
//! 各レイヤは自分より下のレイヤだけに依存する。
//!
//! ```text
//!   temporal_id  (TemporalId / TemporalSet / TemporalMap: 時間の値)
//!        │  葉の値として保持
//!        ▼
//!   tree      … 純空間の永続二分木（FlexTree）。時間を知らない。
//!        │  Combine（値の合成規則）を差し込む
//!        ▼
//!   temporal  … 時空間コア（SpatioTemporalCore）。時間軸の合成を一手に担う。
//!        │  3つの公開の顔
//!        ▼
//!   set / map / table … 公開コレクション
//!        │  を入力・出力として
//!        ▼
//!   query     … Query と演算子（shift / spread / union / …）
//! ```
//!
//! - [`set`]（[`SpatialIdSet`](crate::SpatialIdSet)）: 時空間の集合。
//! - [`map`]（[`SpatialIdMap`](crate::SpatialIdMap)）: 時空間 → 値。
//! - [`table`]（[`SpatialIdTable`](crate::SpatialIdTable)）: 時空間 ⇄ 値の相互検索。
//! - [`traits`] / [`json`]: コレクション共通のトレイトと JSON 直列化。
//! - `testing`: テスト専用の参照実装（本実装のオラクル）。

pub(crate) mod flex_tree;

pub(crate) mod temporal;

pub mod set;

pub mod map;

pub mod table;

pub mod query;

pub mod json;
pub mod traits;

/// 空間主体の時空間集合の参照実装（テスト専用）。
///
/// 本実装は [`SpatialIdSet`](crate::SpatialIdSet) などへ時間ネイティブとして統合済み。
/// このモジュールは統合実装のオラクル（突き合わせ検証）としてテストからのみ使う。
#[cfg(all(test, feature = "temporal_id"))]
pub(crate) mod testing;
