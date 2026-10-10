/// 値や演算子に課す `Send + Sync` の制約
pub mod send_sync;

/// クエリ実行を途中で打ち切るための協調的キャンセル
pub mod cancellation;

/// 演算子の種類
pub mod ops;

/// 演算定義のTrait
pub mod traits;

/// 式全体を見て、最適化し、実行するためのモジュール
pub mod execution;

/// 複数の値が同じ空間で衝突した際の解決ポリシー
pub mod merge_policy;

/// クエリの実行用Trait
pub mod source;

/// クエリの表示の実装
pub mod fmt;

#[doc(hidden)]

pub use execution::Query;
pub use merge_policy::MergePolicy;
