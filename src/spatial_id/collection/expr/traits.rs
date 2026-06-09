use crate::{Error, FlexId, SpatialIdCollection};

/// 空間IDコレクション同士から二項演算を行うTrait。
///
/// 2つのコレクションはいずれも「空間ID → 値」の部分関数であり、ある空間IDにおいて各々は
/// 値を持つ（`Some`）か持たない（`None`）。したがって同じ位置にある2つの空間IDの状態は
/// 次の4つに分かれる。
///
/// | `a` | `b` | 関数 |
/// |-----|-----|------|
/// | `Some` | `Some` | [`both_some`](Self::both_some) |
/// | `Some` | `None` | [`a_only`](Self::a_only) |
/// | `None` | `Some` | [`b_only`](Self::b_only) |
/// | `None` | `None` | そもそも演算を行わない |
///
/// memo:仮に`both_none`関数を作成してしまうと、計算量が膨大になってしまう。
///
/// 入力・出力は [`SpatialIdCollection`] で抽象化されており、
/// `Table` / `Set`、さらに Disk 上の実装に対しても同じ演算が適用できる。
pub trait BinaryOperator<A, B>
where
    A: Ord + PartialEq + Clone,
    B: Ord + PartialEq + Clone,
{
    /// 演算ごとのカスタム設定
    type CustomParameter;

    /// 結果として帰ってくる値の型
    type ResultValue: Ord + PartialEq + Clone;

    fn both_some(
        a: &A,
        b: &B,
        custom_parameter: &Self::CustomParameter,
    ) -> Result<Option<Self::ResultValue>, Error>;

    fn a_only(
        a: &A,
        custom_parameter: &Self::CustomParameter,
    ) -> Result<Option<Self::ResultValue>, Error>;

    fn b_only(
        b: &B,
        custom_parameter: &Self::CustomParameter,
    ) -> Result<Option<Self::ResultValue>, Error>;

    /// 可換な演算か。クエリ最適化での評価順入れ替えの判断に使う。
    fn is_commutative(_custom_parameter: &Self::CustomParameter) -> bool;

    /// コレクション全体の演算。
    ///
    /// 既定実装は2つのコレクションを走査・重なり問い合わせで突き合わせ、各空間を [`both_some`](Self::both_some) / [`a_only`](Self::a_only) / [`b_only`](Self::b_only)へ委譲する汎用ドライバである。入出力のストア種別に依存しない。
    fn execution<SA, SB, O>(
        a: &SA,
        b: &SB,
        custom_parameter: Self::CustomParameter,
    ) -> Result<O, Error>
    where
        SA: SpatialIdCollection<Value = A>,
        SB: SpatialIdCollection<Value = B>,
        O: SpatialIdCollection<Value = Self::ResultValue>,
    {
        let mut result = O::empty();

        for (a_id, a_value) in a.scan() {
            let mut covered: Vec<FlexId> = Vec::new();

            for (overlap, b_value) in b.query(&a_id) {
                if let Some(value) = Self::both_some(&a_value, &b_value, &custom_parameter)? {
                    result.insert(overlap.clone(), value);
                }
                covered.push(overlap);
            }

            for region in subtract_regions(a_id, &covered) {
                if let Some(value) = Self::a_only(&a_value, &custom_parameter)? {
                    result.insert(region, value);
                }
            }
        }

        for (b_id, b_value) in b.scan() {
            let covered: Vec<FlexId> = a.query(&b_id).map(|(id, _)| id).collect();

            for region in subtract_regions(b_id, &covered) {
                if let Some(value) = Self::b_only(&b_value, &custom_parameter)? {
                    result.insert(region, value);
                }
            }
        }

        Ok(result)
    }
}

/// 同一の空間に複数の値が集まったときに、それらを1つの値にする方法。
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

impl<V: Ord> ConflictPolicy<V> {
    /// 既存値 `current`（無ければ `None`）に新しい候補 `incoming` を、方針に従って畳み込む。
    pub fn resolve(&self, current: Option<V>, incoming: V) -> V {
        let Some(current) = current else {
            return incoming;
        };

        match self {
            ConflictPolicy::KeepExisting => current,
            ConflictPolicy::Overwrite => incoming,
            ConflictPolicy::Min => current.min(incoming),
            ConflictPolicy::Max => current.max(incoming),
            ConflictPolicy::Fold(f) => f(&current, &incoming),
        }
    }
}

/// 空間IDコレクションに対して単項演算を行うTrait。
/// 必要な場合は[Self::CustomParameter]に[ConflictPolicy]を含む。
pub trait UnaryOperator<A: Ord + PartialEq + Clone> {
    /// 演算ごとのカスタム設定
    type CustomParameter;

    /// 結果として帰ってくる値の型
    type ResultValue: Ord + PartialEq + Clone;

    /// コレクションに対する単項演算の定義
    fn execution<S, O>(a: &S, custom_parameter: Self::CustomParameter) -> Result<O, Error>
    where
        S: SpatialIdCollection<Value = A>,
        O: SpatialIdCollection<Value = Self::ResultValue>;
}

/// `base` から `holes` の各領域を順に差し引いた、残りの領域の集合を返す。
fn subtract_regions(base: FlexId, holes: &[FlexId]) -> Vec<FlexId> {
    let mut regions = vec![base];

    for hole in holes {
        let mut next = Vec::new();
        for region in regions {
            next.extend(region.difference(hole));
        }
        regions = next;
    }

    regions
}
