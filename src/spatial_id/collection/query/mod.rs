/// 演算子の種類
pub mod ops;

/// 演算定義のTrait
pub mod traits;

/// 式全体を見て、最適化し、実行するためのモジュール
pub mod execution;

/// 複数の値が同じ空間で衝突した際の解決ポリシー
pub mod merge_policy;

/// クエリ実装に共通するユーティリティ関数
pub mod utils;
