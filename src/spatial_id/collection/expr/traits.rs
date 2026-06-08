use crate::SpatialIdTable;

/// 空間IDのTable同士から2項演算を行うTrait
pub trait BinaryOperator<A: Ord + PartialEq + Clone, B: Ord + PartialEq + Clone> {
    /// 演算ごとのカスタム設定
    type CustomParameter;

    /// 結果として帰ってくる値の型
    type ResultValue: Ord + PartialEq + Clone;

    /// Table同士の2項演算の定義
    fn execution(
        a: &SpatialIdTable<A>,
        b: &SpatialIdTable<B>,
        custom_parameter: Self::CustomParameter,
    ) -> SpatialIdTable<Self::ResultValue>;

    /// 可換な演算であるか
    /// aとbを入れ替えても機能するか
    fn is_commutative() -> bool;
}

/// 空間IDのTable同士から単項演算を行うTrait
pub trait UnaryOperator<A: Ord + PartialEq + Clone> {
    /// 演算ごとのカスタム設定
    type CustomParameter;

    /// 結果として帰ってくる値の型
    type ResultValue: Ord + PartialEq + Clone;

    /// Table同士の単項演算の定義
    fn execution(
        a: &SpatialIdTable<A>,
        custom_parameter: Self::CustomParameter,
    ) -> SpatialIdTable<Self::ResultValue>;
}
