pub mod expr;

/// JSON 書き出しで共有する、バックエンド非依存の軽量ユーティリティ。
pub mod json;

/// バックエンド非依存のコレクション抽象（[`SpatialIdCollection`](crate::SpatialIdCollection) など）。
pub mod traits;

/// FlexTree バックエンド（既定）。`morton` feature 有効時は無効化される。
#[cfg(not(feature = "morton"))]
pub mod flex_tree;

/// Morton order バックエンド。`morton` feature 有効時に [`SpatialIdSet`](crate::SpatialIdSet) /
/// [`SpatialIdTable`](crate::SpatialIdTable) の実体となる。
#[cfg(feature = "morton")]
pub mod morton;
