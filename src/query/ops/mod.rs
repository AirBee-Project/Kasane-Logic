///単項演算
pub(crate) mod unary;

/// 値の型を変える変換。型が変わるので `UnaryOperator` ではなく `Source` として実装する。
pub(crate) mod map_values;

/// 二項演算
pub(crate) mod binary;
