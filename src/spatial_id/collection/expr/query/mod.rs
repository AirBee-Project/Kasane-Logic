use alloc::boxed::Box;

pub mod binary;
pub mod unary;

/// 最適化（巡回ドライバと書き換えルール）
mod optimize;

pub use binary::{BinaryKernel, BinaryOp, BinaryOpKernel};
pub use unary::{UnaryKernel, UnaryOp, UnaryOpKernel};

use crate::{Error, SpatialIdCollection};

/// 式全体を表現する型
pub enum Query<C: SpatialIdCollection> {
    /// 演算の起点となるデータ
    Source(C),
    /// 単項演算
    Unary(UnaryOp<C>, Box<Query<C>>),
    /// 二項演算
    Binary(BinaryOp<C>, Box<Query<C>>, Box<Query<C>>),
}

impl<C: SpatialIdCollection> From<C> for Query<C> {
    fn from(collection: C) -> Self {
        collection.into_query()
    }
}

impl<C: SpatialIdCollection> Query<C>
where
    C::Value: 'static,
{
    /// 最適化して[Query]を実行
    pub fn run(self) -> Result<C, Error> {
        match self.optimize() {
            Query::Source(collection) => Ok(collection),
            Query::Unary(op, input) => {
                let input = input.run()?;
                op.run(&input)
            }
            Query::Binary(op, lhs, rhs) => {
                #[cfg(feature = "rayon")]
                let (lhs_res, rhs_res) = rayon::join(|| lhs.run(), || rhs.run());

                #[cfg(not(feature = "rayon"))]
                let (lhs_res, rhs_res) = (lhs.run(), rhs.run());

                op.run(&lhs_res?, &rhs_res?)
            }
        }
    }

    /// 最適化せずに[Query]を実行
    pub fn run_raw(self) -> Result<C, Error> {
        match self {
            Query::Source(collection) => Ok(collection),
            Query::Unary(op, input) => {
                let input = input.run_raw()?;
                op.run(&input)
            }
            Query::Binary(op, lhs, rhs) => {
                #[cfg(feature = "rayon")]
                let (lhs_res, rhs_res) = rayon::join(|| lhs.run_raw(), || rhs.run_raw());

                #[cfg(not(feature = "rayon"))]
                let (lhs_res, rhs_res) = (lhs.run_raw(), rhs.run_raw());

                op.run(&lhs_res?, &rhs_res?)
            }
        }
    }
}
