use super::MergePolicy;
use super::saturating_add::Add;

/// 合計（足し算）を採用するポリシー。
///
/// オーバーフロー時はパニック/ラップアラウンドせず、型が表現できる最大値にクランプします
/// （[`Add`]）。
pub struct Sum;

impl<V: Add> MergePolicy<V> for Sum {
    const IS_COMMUTATIVE: bool = true;

    fn resolve(a: V, b: V) -> V {
        a.saturating_add(b)
    }
}
