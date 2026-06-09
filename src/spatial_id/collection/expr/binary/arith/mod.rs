//! 算術演算（加算ほか）。
//!
//! 集合演算と同じく [`BinaryOperator`](crate::BinaryOperator) の
//! `both_some`/`a_only`/`b_only` を埋めるだけで定義でき、混在ズーム・時間の重なり分割は
//! 既定の `execution` ドライバが担う。重なったセルの値を算術的に合成する点が集合演算と異なる。

/// 加算
pub mod add;

/// 減算
pub mod sub;

/// 乗算
pub mod mul;

/// 算術演算をメソッドとして呼び出す拡張トレイト
pub mod ops;

#[cfg(test)]
mod tests;
