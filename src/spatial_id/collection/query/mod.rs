//! [`Query`]: コレクションに対する遅延演算の式木。
//!
//! [`SpatialIdCollection::query`] で式を組み立て、
//! [`run`](Query::run) で最適化（`optimize`）を経て実行する。
//! 演算子の実体は [`ops`]（unary: shift / stretch / spread / level / fill、
//! binary: set / arith）、演算子定義のトレイトは [`traits`] にある。

/// 演算子の種類
pub mod ops;

/// 演算定義のTrait
pub mod traits;

mod optimize;

use alloc::boxed::Box;

use ops::binary::BinaryOp;
use ops::unary::UnaryOp;

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
        collection.query()
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
                op.run(input)
            }
            Query::Binary(op, lhs, rhs) => {
                #[cfg(feature = "rayon")]
                let (lhs_res, rhs_res) = rayon::join(|| lhs.run(), || rhs.run());

                #[cfg(not(feature = "rayon"))]
                let (lhs_res, rhs_res) = (lhs.run(), rhs.run());

                op.run(lhs_res?, rhs_res?)
            }
        }
    }

    /// 最適化せずに[Query]を実行
    pub fn run_raw(self) -> Result<C, Error> {
        match self {
            Query::Source(collection) => Ok(collection),
            Query::Unary(op, input) => {
                let input = input.run_raw()?;
                op.run(input)
            }
            Query::Binary(op, lhs, rhs) => {
                #[cfg(feature = "rayon")]
                let (lhs_res, rhs_res) = rayon::join(|| lhs.run_raw(), || rhs.run_raw());

                #[cfg(not(feature = "rayon"))]
                let (lhs_res, rhs_res) = (lhs.run_raw(), rhs.run_raw());

                op.run(lhs_res?, rhs_res?)
            }
        }
    }
}
