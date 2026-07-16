use crate::SpatialIdCollection;

pub trait BinaryOperator: Send + Sync {
    /// この演算子に特有のパラメーターを設定する
    type Parameter: Sync;

    /// 演算子の作成
    fn new(parameter: Self::Parameter) -> Self;

    /// コレクションに対する二項演算子の定義
    fn run<T: SpatialIdCollection>(
        &self,
        target_a: &mut T,
        target_b: &T,
    ) -> Result<(), Box<dyn std::error::Error + 'static>>;
}

/// 空間IDコレクションに対して単項演算を行うTrait。
/// 必要な場合は[Self::CustomParameter]に[ConflictPolicy]を含む。
pub trait UnaryOperator: Send + Sync {
    /// この演算子に特有のパラメーターを設定する
    type Parameter: Sync;

    /// 演算子の作成
    fn new(parameter: Self::Parameter) -> Self;

    /// コレクションに対する単項演算の定義
    fn run<T: SpatialIdCollection>(
        &self,
        target: &mut T,
    ) -> Result<(), Box<dyn std::error::Error + 'static>>;
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
