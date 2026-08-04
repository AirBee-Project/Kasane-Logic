use super::Query;
use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::query::execution::group_commutative::types::CommutativityInfo;
use crate::spatial_id::collection::query::traits::UnaryOperator;
use alloc::boxed::Box;
use alloc::vec::Vec;

pub mod types;

impl<V: SafeValue + 'static> Query<V> {
    /// 可換な部分を検知して囲む
    ///
    /// ASTの `Query::Unary` 内に直列に並んだ演算子（`ops`）を走査し、
    /// 互いに可換な連続区間を見つけたら `CommutativeGroup` にラップします。
    pub fn group_commutative_ops(self) -> Self {
        match self {
            Query::Unary(ops, input) => {
                let mut current_ast = input.group_commutative_ops();

                let mut current_group: Vec<Box<dyn UnaryOperator<V>>> = Vec::new();
                let mut current_info: Option<CommutativityInfo> = None;

                let flush = |group: &mut Vec<Box<dyn UnaryOperator<V>>>,
                             info: Option<CommutativityInfo>,
                             ast: &mut Query<V>| {
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

/// [`group_commutative_ops`](Query::group_commutative_ops) +
/// [`sort_commutative_ops`](Query::sort_commutative_ops) と同じ規則で導かれる実行順を、
/// `ops` の所有権を消費せずインデックス列として返す。
///
/// `Query::Unary`/`Query::CommutativeGroup` は所有権を移さないと組み替えられない
/// （`Box<dyn UnaryOperator>` は `Clone` できない）ため、`&Query` しか持てない
/// [`Query::run_on_subset`](super::Query::run_on_subset) 系はこれまで最適化を経由できなかった。
/// この関数は `commutativity_info` / `expansion_ratio` という `&self` メソッドだけを使って
/// 並び順を計算するので、借用のままでも同じ判断ができる。
///
/// `Query::CommutativeGroup` の（全要素が互いに可換という不変条件を持つ）`ops` を渡しても
/// 正しく動く：全ペアが可換なら、このアルゴリズムは自然に1つのクリークとして検出し
/// `expansion_ratio` でソートするだけになる（＝ [`sort_commutative_ops`] と同じ結果）。
/// そのため呼び出し側は `Unary` と `CommutativeGroup` を区別せずこの関数を使ってよい。
pub(crate) fn plan_order<V: SafeValue + 'static>(ops: &[Box<dyn UnaryOperator<V>>]) -> Vec<usize> {
    fn flush<V: SafeValue + 'static>(
        ops: &[Box<dyn UnaryOperator<V>>],
        clique: &mut Vec<usize>,
        info: Option<CommutativityInfo>,
        order: &mut Vec<usize>,
    ) {
        if clique.is_empty() {
            return;
        }
        if clique.len() > 1 && info.is_some_and(|i| i.is_potentially_commutative()) {
            clique.sort_by(|&a, &b| {
                ops[a]
                    .expansion_ratio()
                    .partial_cmp(&ops[b].expansion_ratio())
                    .unwrap_or(core::cmp::Ordering::Equal)
            });
        }
        order.append(clique);
    }

    let mut order = Vec::with_capacity(ops.len());
    let mut clique: Vec<usize> = Vec::new();
    let mut clique_info: Option<CommutativityInfo> = None;

    for (i, op) in ops.iter().enumerate() {
        let info = op.commutativity_info();

        let mut is_clique = false;
        if let Some(cur_info) = clique_info
            && cur_info.is_potentially_commutative()
            && info.is_potentially_commutative()
        {
            is_clique = clique
                .iter()
                .all(|&existing| ops[existing].commutativity_info().can_commute_with(&info));
        }

        if is_clique
            || (clique_info.is_some_and(|c| !c.is_potentially_commutative())
                && !info.is_potentially_commutative())
        {
            clique.push(i);
        } else {
            flush(ops, &mut clique, clique_info, &mut order);
            clique.push(i);
            clique_info = Some(info);
        }
    }
    flush(ops, &mut clique, clique_info, &mut order);
    order
}

#[cfg(test)]
mod test;
