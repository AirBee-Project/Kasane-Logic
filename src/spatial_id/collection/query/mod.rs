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

/// クエリの作業表現（`WorkingTree`）
pub mod working;

/// クエリの表示の実装
pub mod fmt;

/// 分離可能演算のための一様ズームの平坦表現
///
/// クエリエンジンの実装詳細。`UnaryOperator::grid_op` の戻り値型を名指しできるよう
/// 公開しているだけで、中身は不透明。ドキュメントには出さない。
#[doc(hidden)]
pub mod grid;

pub use execution::Query;
pub use merge_policy::MergePolicy;
pub use working::WorkingTree;
