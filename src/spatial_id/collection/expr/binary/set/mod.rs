//! 集合演算（和・積・差・対称差）と、その一般化である値合成。
//!
//! いずれも [`BinaryOperator`](crate::BinaryOperator) の `both_some`/`a_only`/`b_only` を
//! 埋めるだけで定義でき、混在ズーム・時間の重なり分割は既定の `execution` ドライバが担う。

/// 和集合
pub mod union;

/// 積集合
pub mod intersection;

/// 差集合
pub mod difference;

/// 対称差
pub mod symmetric_difference;

/// マスク（積・左値保持）
pub mod mask;

/// 異型の値合成（4演算の一般化）
pub mod combine;

/// 集合演算をメソッドとして呼び出す拡張トレイト
pub mod ops;

#[cfg(test)]
mod tests;
