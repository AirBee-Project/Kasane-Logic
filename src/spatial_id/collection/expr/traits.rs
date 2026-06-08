use crate::{Error, SpatialIdTable};

/// 空間IDのTable同士から2項演算を行うTrait。
///
/// 2つのTableはいずれも「空間ID → 値」の部分関数であり、ある空間IDにおいて各Tableは値を持つ（`Some`）か持たない（`None`）。したがって同じ位置にある2つの空間IDの状態は次の4つに分かれる。
///
/// | `a` | `b` | 関数 |
/// |-----|-----|------|
/// | `Some` | `Some` | [`both_some`](Self::both_some) |
/// | `Some` | `None` | [`a_only`](Self::a_only) |
/// | `None` | `Some` | [`b_only`](Self::b_only) |
/// | `None` | `None` | そもそも演算を行わない |
///
/// memo:仮に`both_none`関数を作成してしまうと、計算量が膨大になってしまう。
pub trait BinaryOperator<A, B>
where
    A: Ord + PartialEq + Clone,
    B: Ord + PartialEq + Clone,
{
    /// 演算ごとのカスタム設定
    type CustomParameter;

    /// 結果として帰ってくる値の型
    type ResultValue: Ord + PartialEq + Clone;

    /// 両方の空間IDが値を持つ場合
    fn both_some(
        a: &A,
        b: &B,
        custom_parameter: &Self::CustomParameter,
    ) -> Result<Option<Self::ResultValue>, Error>;

    /// `a` の空間IDのみが値を持つ場合
    fn a_only(
        a: &A,
        custom_parameter: &Self::CustomParameter,
    ) -> Result<Option<Self::ResultValue>, Error>;

    /// `b` の空間IDのみが値を持つ場合
    fn b_only(
        b: &B,
        custom_parameter: &Self::CustomParameter,
    ) -> Result<Option<Self::ResultValue>, Error>;

    /// 可換な演算か。
    fn is_commutative(_custom_parameter: &Self::CustomParameter) -> bool;

    /// Table全体の演算。
    fn execution(
        _a: &SpatialIdTable<A>,
        _b: &SpatialIdTable<B>,
        _custom_parameter: Self::CustomParameter,
    ) -> Result<SpatialIdTable<Self::ResultValue>, Error> {
        todo!()
    }
}

/// 同一の空間セルに複数の値が集まったときに、それらを1つの値へ畳み込む方法。
pub enum ConflictPolicy<V> {
    /// 既存の値を保持し、後から来た候補を捨てる。
    KeepExisting,
    /// 後から来た候補で既存の値を上書きする。
    Overwrite,
    /// [`Ord`] 上で小さい方を採用する。
    Min,
    /// [`Ord`] 上で大きい方を採用する。
    Max,
    /// ユーザ定義の関数で合成する。引数は `(既存値, 新しい候補)` の順。
    Fold(fn(&V, &V) -> V),
}

/// 空間IDのTableに対して単項演算を行うTrait。
/// 必要な場合は[Self::CustomParameter]に[ConflictPolicy]を含む
pub trait UnaryOperator<A: Ord + PartialEq + Clone> {
    /// 演算ごとのカスタム設定
    type CustomParameter;

    /// 結果として帰ってくる値の型
    type ResultValue: Ord + PartialEq + Clone;

    /// Tableに対する単項演算の定義
    fn execution(
        a: &SpatialIdTable<A>,
        custom_parameter: Self::CustomParameter,
    ) -> Result<SpatialIdTable<Self::ResultValue>, Error>;
}
