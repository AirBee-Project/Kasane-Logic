/// 演算子の種類
pub mod ops;

/// 演算定義のTrait
pub mod traits;

/// 式全体を見て、最適化し、実行するためのモジュール（領域限定の遅延評価を含む）
pub mod execution;

/// 複数の値が同じ空間で衝突した際の解決ポリシー
pub mod merge_policy;

/// クエリの入力源（`Source`）
pub mod source;

/// クエリの表示の実装
pub mod fmt;

pub use execution::Query;
pub use merge_policy::MergePolicy;
