/// クエリ実行を途中で打ち切るための協調的キャンセル
pub mod cancellation;

/// 演算子の種類
pub mod ops;

/// 複数の値が同じ空間で衝突した際の解決ポリシー
pub mod merge_policy;
pub use merge_policy::MergePolicy;

use crate::{
    Error, FlexId, RangeId, SafeValue, spatial_id::collection::flex_tree::core::ptr::MaybeSendSync,
};
use alloc::{boxed::Box, vec, vec::Vec};
use cancellation::CancellationToken;

/// `(FlexId, V)` を1件ずつ生成するイテレーター。
pub type ValueIter<'a, V> = Box<dyn Iterator<Item = (FlexId, V)> + 'a>;

/// Query全体を表現する型。
pub enum Query<V: SafeValue + 'static> {
    /// 演算の起点となる[Source]
    Source(Box<dyn Source<Value = V>>),

    /// 連続した単項演算子
    Unary(Vec<Box<dyn UnaryOperator<V>>>, Box<Query<V>>),

    // 二項演算
    Binary(Box<dyn BinaryOperator<V>>, Box<Query<V>>, Box<Query<V>>),

    /// エラー状態
    Error(Error),
}

impl<V: SafeValue + 'static> Query<V> {
    /// 検証してから実行し、結果を`(FlexId, V)`のペアとして1件ずつ生成するイテレーターを返す。
    pub fn run(&self) -> Result<ValueIter<'_, V>, Error> {
        self.run_within_cancellable(RangeId::everything(), CancellationToken::never())
    }

    /// [`run`](Self::run) の[CancellationToken]で打ち切り可能な版。
    pub fn run_cancellable(&self, token: CancellationToken) -> Result<ValueIter<'_, V>, Error> {
        self.run_within_cancellable(RangeId::everything(), token)
    }

    /// `target` と交差する部分だけを検証してから評価する。
    ///
    /// 演算子は [`UnaryOperator::inverse_bounds`]/[`BinaryOperator::inverse_bounds`] を使って
    /// 「`target` を得るのに実際どこまで入力が必要か」を逆算しながら木を降りるため、
    /// falloff/shiftのように近傍を参照する演算でも、対象領域の外にある無関係な入力を
    /// 読み込まずに済む。
    pub fn run_within(&self, target: impl Into<RangeId>) -> Result<ValueIter<'_, V>, Error> {
        self.run_within_cancellable(target, CancellationToken::never())
    }

    /// [`run_within`](Self::run_within) の[CancellationToken]で打ち切り可能な版。
    pub fn run_within_cancellable(
        &self,
        target: impl Into<RangeId>,
        token: CancellationToken,
    ) -> Result<ValueIter<'_, V>, Error> {
        self.validate()?;
        self.run_within_unchecked(target.into(), &token)
    }

    /// [`run_within`](Self::run_within) の本体。事前に [`validate`](Self::validate) 済みであることを前提とする。
    fn run_within_unchecked(
        &self,
        target: RangeId,
        token: &CancellationToken,
    ) -> Result<ValueIter<'_, V>, Error> {
        if token.is_cancelled() {
            return Err(Error::Cancelled);
        }
        match self {
            Query::Source(source) => source.get(target, token.clone()),
            Query::Unary(ops, input) => {
                let mut targets: Vec<RangeId> = Vec::with_capacity(ops.len());
                let mut req = target;
                let mut reachable = true;
                for op in ops.iter().rev() {
                    targets.push(req.clone());
                    match op.inverse_bounds(req.clone()) {
                        Some(t) => req = t,
                        None => {
                            reachable = false;
                            break;
                        }
                    }
                }
                if !reachable {
                    return Ok(Box::new(core::iter::empty()));
                }
                targets.reverse();

                let mut iter = input.run_within_unchecked(req, token)?;
                for (op, op_target) in ops.iter().zip(targets) {
                    if token.is_cancelled() {
                        return Err(Error::Cancelled);
                    }
                    iter = op.run(iter, op_target, token.clone())?;
                }
                Ok(iter)
            }
            Query::Binary(op, lhs, rhs) => {
                let (l, r) = op.inverse_bounds(target);
                let lhs_iter = match l {
                    Some(t) => lhs.run_within_unchecked(t, token)?,
                    None => Box::new(core::iter::empty()),
                };
                let rhs_iter = match r {
                    Some(t) => rhs.run_within_unchecked(t, token)?,
                    None => Box::new(core::iter::empty()),
                };
                op.run(lhs_iter, rhs_iter, token.clone())
            }
            Query::Error(e) => Err(e.clone()),
        }
    }

    /// `self` を単項演算子で包む。
    pub(crate) fn wrap_unary<O>(self, op: O) -> Self
    where
        O: UnaryOperator<V> + 'static,
    {
        match self {
            Query::Unary(mut ops, input) => {
                ops.push(Box::new(op));
                Query::Unary(ops, input)
            }
            other => Query::Unary(
                vec![Box::new(op) as Box<dyn UnaryOperator<V>>],
                Box::new(other),
            ),
        }
    }

    /// 実行までに全てのQueryのパラメーターが正常値の範囲内か検証する
    pub fn validate(&self) -> Result<(), Error> {
        match self {
            Query::Source(_) => Ok(()),
            Query::Unary(ops, input) => {
                input.validate()?;
                for op in ops {
                    op.validate()?;
                }
                Ok(())
            }
            Query::Binary(op, lhs, rhs) => {
                lhs.validate()?;
                rhs.validate()?;
                op.validate()
            }
            Query::Error(e) => Err(e.clone()),
        }
    }
}

/// 二項演算子の定義。
pub trait BinaryOperator<V: SafeValue>: MaybeSendSync {
    /// パラメーターの事前検証
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }

    /// `lhs` と `rhs` を1つに合成した結果を流すイテレーターを作る。
    fn run<'a>(
        &'a self,
        lhs: ValueIter<'a, V>,
        rhs: ValueIter<'a, V>,
        token: CancellationToken,
    ) -> Result<ValueIter<'a, V>, Error>;

    /// 出力領域 `output` を得るのに必要な、左右それぞれの入力領域を逆算する。
    fn inverse_bounds(&self, output: RangeId) -> (Option<RangeId>, Option<RangeId>) {
        (Some(output.clone()), Some(output))
    }
}

/// 単項演算子の定義。
pub trait UnaryOperator<V: SafeValue>: MaybeSendSync {
    /// パラメーターの事前検証
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }

    /// `input` にこの演算を適用した結果を流すイテレーターを作る。
    fn run<'a>(
        &'a self,
        input: ValueIter<'a, V>,
        target: RangeId,
        token: CancellationToken,
    ) -> Result<ValueIter<'a, V>, Error>;

    /// 出力領域 `output` を得るのに必要な入力領域を逆算する。
    fn inverse_bounds(&self, output: RangeId) -> Option<RangeId> {
        Some(output)
    }
}

/// クエリを実行するためのTrait。
pub trait Source: MaybeSendSync {
    type Value: SafeValue;

    /// `target` と交差する部分の[FlexId]と値の組を取り出す
    fn get<'a>(
        &'a self,
        target: RangeId,
        token: CancellationToken,
    ) -> Result<ValueIter<'a, Self::Value>, Error>;

    fn query(self) -> Query<Self::Value>
    where
        Self: Sized + 'static,
    {
        Query::Source(Box::new(self))
    }
}

/// `Source` を実装する型を、二項演算子の引数などで直接 [`Query`] として渡せるようにする。
impl<V: SafeValue + 'static, S: Source<Value = V> + 'static> From<S> for Query<V> {
    fn from(source: S) -> Self {
        source.query()
    }
}
