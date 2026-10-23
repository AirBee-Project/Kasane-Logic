///単項演算
pub(crate) mod unary;

/// 値、または FlexId と値を見た変換と絞り込み。値の型が変わりうるので [`UnaryOperator`](crate::query::ops::unary::UnaryOperator) ではなく [`Source`](crate::query::Source) として実装する。
pub(crate) mod transform;

/// 二項演算
pub(crate) mod binary;
