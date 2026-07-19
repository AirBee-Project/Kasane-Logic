pub mod average;
pub mod keep_existing;
pub mod max;
pub mod min;
pub mod overwrite;
pub mod sum;

pub use average::Average;
pub use keep_existing::KeepExisting;
pub use max::Max;
pub use min::Min;
pub use overwrite::Overwrite;
pub use sum::Sum;

use crate::spatial_id::collection::flex_tree::core::ptr::MaybeSendSync;

/// 同一の空間に複数の値が集まったときに、それらを1つの値に合成するポリシー。
/// 関数ポインタを用いず、型によって静的にディスパッチされるため高速です。
pub trait MergePolicy<V>: MaybeSendSync + 'static {
    /// このポリシーが可換（`resolve(a, b) == resolve(b, a)`）であるかどうかを示します。
    /// ASTの最適化（演算の順序入れ替えなど）に利用されます。
    const IS_COMMUTATIVE: bool;

    /// 衝突した2つの値を合成して返す（インクリメンタルな操作用）
    /// ※平均などを実装する場合、これは「移動平均」になります。
    fn resolve(a: V, b: V) -> V;

    /// 衝突した複数の値を一括で合成して返す（集約演算用）
    /// 平均などのポリシーは、このメソッドをオーバーライドすることで
    /// 正確な計算（例：合計 / 要素数）を行うことができます。
    fn resolve_many(mut iter: impl ExactSizeIterator<Item = V>) -> Option<V> {
        let first = iter.next()?;
        Some(iter.fold(first, Self::resolve))
    }
}
