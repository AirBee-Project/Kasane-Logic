pub mod keep_existing;
pub mod max;
pub mod min;
pub mod overwrite;
pub mod sum;

pub use keep_existing::KeepExisting;
pub use max::Max;
pub use min::Min;
pub use overwrite::Overwrite;
pub use sum::Sum;

/// 同一の空間に複数の値が集まったときに、それらを1つの値に合成するポリシー。
/// 関数ポインタを用いず、型によって静的にディスパッチされるため高速です。
pub trait MergePolicy<V>: Send + Sync + 'static {
    /// このポリシーが可換（`resolve(a, b) == resolve(b, a)`）であるかどうかを示します。
    /// ASTの最適化（演算の順序入れ替えなど）に利用されます。
    const IS_COMMUTATIVE: bool;

    /// 衝突した2つの値を合成して返す
    fn resolve(a: V, b: V) -> V;
}
