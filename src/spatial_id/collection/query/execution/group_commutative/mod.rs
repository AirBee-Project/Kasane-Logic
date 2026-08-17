use super::Query;
use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::query::traits::UnaryOperator;
use alloc::boxed::Box;
use alloc::vec::Vec;
use runs::UnaryOperatorSliceExt;

pub mod runs;
pub mod types;

impl<V: SafeValue + 'static> Query<V> {
    /// 可換な部分を検知して囲む
    ///
    /// ASTの `Query::Unary` 内に直列に並んだ演算子（`ops`）を走査し、互いに可換な連続区間を見つけたら `CommutativeGroup` にラップする。
    pub fn group_commutative_ops(self) -> Self {
        match self {
            Query::Unary(ops, input) => {
                let runs = ops.commutative_runs();
                let mut ast = input.group_commutative_ops();
                let mut remaining = ops.into_iter();

                for run in runs {
                    let group: Vec<Box<dyn UnaryOperator<V>>> =
                        remaining.by_ref().take(run.range.len()).collect();
                    ast = if run.sortable {
                        Query::CommutativeGroup(run.info, group, Box::new(ast))
                    } else {
                        Query::Unary(group, Box::new(ast))
                    };
                }
                ast
            }
            Query::CommutativeGroup(info, ops, input) => {
                Query::CommutativeGroup(info, ops, Box::new(input.group_commutative_ops()))
            }
            Query::Binary(op, lhs, rhs) => Query::Binary(
                op,
                Box::new(lhs.group_commutative_ops()),
                Box::new(rhs.group_commutative_ops()),
            ),
            other => other,
        }
    }
}

#[cfg(test)]
mod test;
