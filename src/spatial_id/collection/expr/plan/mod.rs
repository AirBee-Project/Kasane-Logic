use alloc::boxed::Box;

pub mod binary;
pub mod unary;

pub use binary::{BinaryKernel, BinaryOp, BinaryOpKernel};
pub use unary::{UnaryKernel, UnaryOp, UnaryOpKernel};

use crate::{Error, SpatialIdCollection};

/// メソッドチェーン全体を表す単一のデータ型
pub enum Plan<C: SpatialIdCollection> {
    /// 葉：演算の起点となるコレクション。
    Source(C),
    /// 単項演算ノード。
    Unary(UnaryOp<C>, Box<Plan<C>>),
    /// 二項演算ノード。
    Binary(BinaryOp<C>, Box<Plan<C>>, Box<Plan<C>>),
}

impl<C: SpatialIdCollection> Plan<C>
where
    C::Value: 'static,
{
    /// 最適化を行う
    pub fn optimize(self) -> Self {
        self
    }

    /// 最適化せず、プランをそのまま実行
    pub fn execution(self) -> Result<C, Error> {
        match self {
            Plan::Source(collection) => Ok(collection),
            Plan::Unary(op, input) => {
                let input = input.execution()?;
                op.run(&input)
            }
            Plan::Binary(op, lhs, rhs) => {
                let lhs = lhs.execution()?;
                let rhs = rhs.execution()?;
                op.run(&lhs, &rhs)
            }
        }
    }

    /// 最適化してから実行
    pub fn optimized_execution(self) -> Result<C, Error> {
        self.optimize().execution()
    }
}
