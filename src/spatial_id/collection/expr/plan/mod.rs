use alloc::boxed::Box;

pub mod binary;
pub mod unary;

/// 最適化（巡回ドライバと書き換えルール）
mod optimize;

pub use binary::{BinaryKernel, BinaryOp, BinaryOpKernel};
pub use unary::{UnaryKernel, UnaryOp, UnaryOpKernel};

use crate::{Error, SpatialIdCollection};

/// 式全体を表現する型
pub enum Plan<C: SpatialIdCollection> {
    /// 演算の起点となるデータ
    Source(C),
    /// 単項演算
    Unary(UnaryOp<C>, Box<Plan<C>>),
    /// 二項演算
    Binary(BinaryOp<C>, Box<Plan<C>>, Box<Plan<C>>),
}

impl<C: SpatialIdCollection> From<C> for Plan<C> {
    fn from(collection: C) -> Self {
        collection.plan()
    }
}

impl<C: SpatialIdCollection> Plan<C>
where
    C::Value: 'static,
{
    /// 最適化して式を実行
    pub fn execution(self) -> Result<C, Error> {
        match self.optimize() {
            Plan::Source(collection) => Ok(collection),
            Plan::Unary(op, input) => {
                let input = input.execution()?;
                op.run(&input)
            }
            Plan::Binary(op, lhs, rhs) => {
                #[cfg(feature = "rayon")]
                let (lhs_res, rhs_res) = rayon::join(|| lhs.execution(), || rhs.execution());

                #[cfg(not(feature = "rayon"))]
                let (lhs_res, rhs_res) = (lhs.execution(), rhs.execution());

                op.run(&lhs_res?, &rhs_res?)
            }
        }
    }

    /// 最適化せずに式を実行
    pub fn unoptimized_execution(self) -> Result<C, Error> {
        self.optimize().execution()
    }
}
