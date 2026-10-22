use super::MergePolicy;
use super::saturating_add::SaturatingAdd;

/// 足し算を行う[MergePolicy]。
///
/// オーバーフロー時はパニック/ラップアラウンドせず、型が表現できる最大値となる。
pub struct Sum;

impl<V: SaturatingAdd> MergePolicy<V> for Sum {
    const IS_COMMUTATIVE: bool = true;

    fn resolve(a: V, b: V) -> V {
        a.saturating_add(b)
    }
}
