use alloc::boxed::Box;

use crate::spatial_id::collection::expr::plan::binary::BinaryOp;
use crate::spatial_id::collection::expr::plan::unary::UnaryOp;
use crate::{Error, SpatialIdCollection};

pub mod binary;
pub mod unary;

/// メソッドチェーン全体を表す単一のデータ型（再帰的 AST）。
///
/// 葉から結果まで、ひとつのコレクション型 `C` が一貫して流れる。
pub enum Plan<C: SpatialIdCollection> {
    /// 葉：演算の起点となるコレクション。
    Source(C),
    /// 単項演算ノード。
    Unary { op: UnaryOp<C>, input: Box<Plan<C>> },
    /// 二項演算ノード。
    Binary {
        op: BinaryOp<C>,
        lhs: Box<Plan<C>>,
        rhs: Box<Plan<C>>,
    },
}

impl<C: SpatialIdCollection> Plan<C> {
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
