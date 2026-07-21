pub mod average;
pub mod difference;
pub mod keep_existing;
pub mod max;
pub mod min;
pub mod overwrite;
pub mod saturating_add;
pub mod sum;

pub use average::Average;
pub use difference::Difference;
pub use keep_existing::KeepExisting;
pub use max::Max;
pub use min::Min;
pub use overwrite::Overwrite;
pub use sum::Sum;

use crate::spatial_id::collection::flex_tree::core::ptr::MaybeSendSync;

/// 同一の空間に複数の値が集まったときに、それらを1つの値に集約する規則
pub trait MergePolicy<V>: MaybeSendSync + 'static {
    /// この集約規則が可換（`resolve(a, b) == resolve(b, a)`）であるかどうか
    const IS_COMMUTATIVE: bool;

    /// このポリシーの名前を示す文字列定数。
    const NAME: &'static str;

    /// 衝突した2つの値を合成して返す
    fn resolve(a: V, b: V) -> V;

    /// 衝突した複数の値を一括で合成して返す
    /// 必要であればオーバーロードして平均や最頻値などを実装できる
    fn resolve_many(mut iter: impl ExactSizeIterator<Item = V>) -> Option<V> {
        let first = iter.next()?;
        Some(iter.fold(first, Self::resolve))
    }
}
