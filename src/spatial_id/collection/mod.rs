pub mod expr;
pub mod flex_tree;
/// 空間主体の時空間集合の参照実装（テスト専用）。
///
/// 本実装は [`SpatialIdSet`](crate::SpatialIdSet) に時間ネイティブとして統合済み。
/// このモジュールは統合実装のオラクル（突き合わせ検証）としてテストからのみ使う。
#[cfg(all(test, feature = "temporal_id"))]
pub(crate) mod spatio_temporal;
