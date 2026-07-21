use super::Query;
use crate::SpatialIdCollection;
use crate::spatial_id::collection::query::execution::group_commutative::types::CommutativityInfo;
use crate::spatial_id::collection::query::traits::UnaryOperator;
use alloc::boxed::Box;
use alloc::vec::Vec;

pub mod types;

impl<S: SpatialIdCollection> Query<S>
where
    S::Value: 'static,
{
    /// 可換な部分を検知して囲む
    ///
    /// ASTの `Query::Unary` 内に直列に並んだ演算子（`ops`）を走査し、
    /// 互いに可換な連続区間を見つけたら `CommutativeGroup` にラップします。
    pub fn group_commutative_ops(self) -> Self
    where
        S::Working: 'static,
    {
        match self {
            Query::Unary(ops, input) => {
                let mut current_ast = input.group_commutative_ops();

                let mut current_group: Vec<Box<dyn UnaryOperator<S::Working>>> = Vec::new();
                let mut current_info: Option<CommutativityInfo> = None;

                let flush = |group: &mut Vec<Box<dyn UnaryOperator<S::Working>>>,
                             info: Option<CommutativityInfo>,
                             ast: &mut Query<S>| {
                    if group.is_empty() {
                        return;
                    }
                    let inner = core::mem::replace(
                        ast,
                        Query::Error(crate::Error::SpatialId(
                            crate::error::SpatialIdError::ZOutOfRange { z: 255 },
                        )),
                    );
                    if group.len() > 1
                        && let Some(info_val) = info
                        && info_val.is_potentially_commutative()
                    {
                        // 動的ソートは実行時(Query::execute内)で行うため、ここではソートせず順序を維持する
                        *ast = Query::CommutativeGroup(
                            info_val,
                            core::mem::take(group),
                            Box::new(inner),
                        );
                    } else {
                        *ast = Query::Unary(core::mem::take(group), Box::new(inner));
                    }
                };

                for op in ops {
                    let info = op.commutativity_info();

                    let mut is_clique = false;
                    if let Some(cur_info) = current_info
                        && cur_info.is_potentially_commutative()
                        && info.is_potentially_commutative()
                    {
                        is_clique = current_group.iter().all(|existing_op| {
                            existing_op.commutativity_info().can_commute_with(&info)
                        });
                    }

                    if is_clique
                        || (current_info.is_some_and(|c| !c.is_potentially_commutative())
                            && !info.is_potentially_commutative())
                    {
                        current_group.push(op);
                    } else {
                        flush(&mut current_group, current_info, &mut current_ast);
                        current_group.push(op);
                        current_info = Some(info);
                    }
                }
                flush(&mut current_group, current_info, &mut current_ast);
                current_ast
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
