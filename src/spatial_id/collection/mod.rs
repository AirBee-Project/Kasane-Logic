pub(crate) mod flex_tree;
pub(crate) mod temporal;

pub mod set;

pub mod map;

pub mod table;

pub mod query;

pub mod serde_impl;
pub mod traits;

/// 空間主体の時空間集合の参照実装
/// テスト以外では使用しない
#[cfg(all(test, feature = "temporal_id"))]
pub(crate) mod testing;
