use alloc::boxed::Box;
use core::marker::PhantomData;

use crate::spatial_id::collection::expr::plan::binary::{BinaryKernel, BinaryOpKernel};
use crate::spatial_id::collection::expr::plan::unary::{UnaryKernel, UnaryOpKernel};
use crate::{BinaryOperator, Error, SpatialIdCollection, UnaryOperator};

pub mod binary;
pub mod unary;

/// メソッドチェーン全体を表す単一のデータ型（再帰的 AST）。
///
/// 葉から結果まで、ひとつのコレクション型 `C` が一貫して流れる。
pub enum Plan<C: SpatialIdCollection> {
    /// 葉：演算の起点となるコレクション。
    Source(C),
    /// 単項演算ノード。
    Unary {
        op: Box<dyn UnaryKernel<C>>,
        input: Box<Plan<C>>,
    },
    /// 二項演算ノード。
    Binary {
        op: Box<dyn BinaryKernel<C>>,
        lhs: Box<Plan<C>>,
        rhs: Box<Plan<C>>,
    },
}

impl<C: SpatialIdCollection> Plan<C>
where
    C::Value: 'static,
{
    /// 任意の単項演算を適用する（内部ヘルパー）。
    pub fn apply_unary<Op>(self, param: <Op as UnaryOperator<C::Value>>::CustomParameter) -> Self
    where
        Op: UnaryOperator<C::Value, ResultValue = C::Value> + 'static,
        <Op as UnaryOperator<C::Value>>::CustomParameter: 'static,
    {
        let kernel = Box::new(UnaryOpKernel::<Op, _> {
            param,
            _op: PhantomData,
        });
        Plan::Unary {
            op: kernel,
            input: Box::new(self),
        }
    }

    /// 任意の二項演算を適用する（内部ヘルパー）。
    pub fn apply_binary<Op>(
        self,
        other: Self,
        param: <Op as BinaryOperator<C::Value, C::Value>>::CustomParameter,
    ) -> Self
    where
        Op: BinaryOperator<C::Value, C::Value, ResultValue = C::Value> + 'static,
        <Op as BinaryOperator<C::Value, C::Value>>::CustomParameter: 'static,
    {
        let kernel = Box::new(BinaryOpKernel::<Op, _> {
            param,
            _op: PhantomData,
        });
        Plan::Binary {
            op: kernel,
            lhs: Box::new(self),
            rhs: Box::new(other),
        }
    }

    // =========================================================================
    // 評価・最適化
    // =========================================================================

    /// 最適化を行う
    pub fn optimize(self) -> Self {
        self
    }

    /// 最適化せず、プランをそのまま実行して結果を材化する。
    pub fn execution(self) -> Result<C, Error> {
        match self {
            Plan::Source(collection) => Ok(collection),
            Plan::Unary { op, input } => {
                let input = input.execution()?;
                op.run(&input)
            }
            Plan::Binary { op, lhs, rhs } => {
                let lhs = lhs.execution()?;
                let rhs = rhs.execution()?;
                op.run(&lhs, &rhs)
            }
        }
    }

    /// 最適化してから実行する。
    pub fn optimized_execution(self) -> Result<C, Error> {
        self.optimize().execution()
    }
}
