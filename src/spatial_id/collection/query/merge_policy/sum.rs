use super::MergePolicy;
use super::saturating_add::Add;

/// 足し算を行う[MergePolicy]。
///
/// オーバーフロー時はパニック/ラップアラウンドせず、型が表現できる最大値となります。
pub struct Sum;

impl<V: Add> MergePolicy<V> for Sum {
    const IS_COMMUTATIVE: bool = true;
    const NAME: &'static str = "Sum";

    fn resolve(a: V, b: V) -> V {
        a.saturating_add(b)
    }
}
