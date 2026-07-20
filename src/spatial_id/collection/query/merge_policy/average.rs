use super::MergePolicy;
use super::saturating_add::Add;
use core::ops::Div;

/// 平均（Average）を採用するポリシー。
///
/// `ZoomOut` などの一括集約操作では `resolve_many` が呼ばれ、真の平均（Sum / Count）が計算されます。
/// 逐次挿入時（`resolve`）では `(a + b) / 2` の移動平均となることに注意してください。
///
/// 合計はオーバーフロー時にパニック/ラップアラウンドせず型の最大値にクランプします（[`Add`]）。
/// 要素数を `V` へ変換する境界は `From<u16>`（既存互換）のままなので、65536件を超える集約では
/// `count as u16` が切り詰められ平均が狂います。`i32` のような符号付き整数は `From<u32>` を
/// 実装しないため、境界を広げると既存の呼び出し側を壊してしまいます。
pub struct Average;

impl<V> MergePolicy<V> for Average
where
    V: Add + Div<V, Output = V> + From<u16> + Send + Sync + 'static,
{
    const IS_COMMUTATIVE: bool = true;

    fn resolve(a: V, b: V) -> V {
        a.saturating_add(b) / V::from(2u16)
    }

    fn resolve_many(mut iter: impl ExactSizeIterator<Item = V>) -> Option<V> {
        let count = iter.len();
        let first = iter.next()?;
        let sum = iter.fold(first, Add::saturating_add);
        let count_v = V::from(count as u16);
        Some(sum / count_v)
    }
}
