use super::traits::{BinaryOperator, UnaryOperator};
use crate::Error;
use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::query::cancellation::CancellationToken;
use crate::spatial_id::collection::query::execution::group_commutative::runs::UnaryOperatorSliceExt;
use crate::spatial_id::collection::query::execution::group_commutative::types::CommutativityInfo;
use crate::spatial_id::collection::query::source::Source;
use crate::spatial_id::collection::query::working::WorkingTree;
use crate::trace::trace_span;
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

pub mod composed_chain;
pub mod group_commutative;

#[cfg(test)]
mod test;

/// Query全体を表現する型。
pub enum Query<V: SafeValue + 'static> {
    /// 演算の起点
    Source(Box<dyn Source<Value = V>>),

    /// 連続した単項演算子
    Unary(Vec<Box<dyn UnaryOperator<V>>>, Box<Query<V>>),

    /// 互いに可換な単項演算子のグループ
    CommutativeGroup(
        CommutativityInfo,
        Vec<Box<dyn UnaryOperator<V>>>,
        Box<Query<V>>,
    ),

    // 二項演算
    Binary(Box<dyn BinaryOperator<V>>, Box<Query<V>>, Box<Query<V>>),

    /// エラー状態を保持
    Error(Error),
}

// Queryの構築
impl<V: SafeValue + 'static> Query<V> {
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
}

//Queryの検証と最適化
impl<V: SafeValue + 'static> Query<V> {
    /// 実行までに全てのQueryのパラメーターが正常値の範囲内か検証する。
    pub fn validate(&self) -> Result<(), Error> {
        match self {
            Query::Source(_) => Ok(()),
            Query::Unary(ops, input) | Query::CommutativeGroup(_, ops, input) => {
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

    /// AST最適化を適用する（実行は行わない）。
    pub fn optimize(self) -> Self {
        self.group_commutative_ops().sort_commutative_ops()
    }

    /// 可換グループ内の演算子を拡大率が小さい順へ並び替える。
    pub fn sort_commutative_ops(self) -> Self {
        match self {
            Query::CommutativeGroup(info, mut ops, input) => {
                ops.sort_by(|a, b| {
                    a.expansion_ratio()
                        .partial_cmp(&b.expansion_ratio())
                        .unwrap_or(core::cmp::Ordering::Equal)
                });
                Query::CommutativeGroup(info, ops, Box::new(input.sort_commutative_ops()))
            }
            Query::Unary(ops, input) => Query::Unary(ops, Box::new(input.sort_commutative_ops())),
            Query::Binary(op, lhs, rhs) => Query::Binary(
                op,
                Box::new(lhs.sort_commutative_ops()),
                Box::new(rhs.sort_commutative_ops()),
            ),
            other => other,
        }
    }
}

// Queryの全体実行
impl<V: SafeValue + 'static> Query<V> {
    /// 検証・AST最適化を適用して実行し、[WorkingTree]のまま返す。
    pub fn run_working_tree(self) -> Result<WorkingTree<V>, Error> {
        self.validate()?;
        self.optimize().raw_run_working_tree()
    }

    /// 検証も最適化もせず [`Query`] を実行し、[WorkingTree]のまま返す。
    pub fn raw_run_working_tree(self) -> Result<WorkingTree<V>, Error> {
        fn run_internal_flat<V: SafeValue + 'static>(
            query: Query<V>,
            token: &CancellationToken,
        ) -> Result<alloc::vec::Vec<(crate::FlexId, V)>, Error> {
            match query {
                Query::Source(source) => source.read_all_flat(token),
                Query::Unary(ops, input) | Query::CommutativeGroup(_, ops, input) => {
                    let order: Vec<&dyn UnaryOperator<V>> = ops.iter().map(|op| &**op).collect();
                    composed_chain::run_composed_chain_flat(
                        &order,
                        run_internal_flat(*input, token)?,
                        token,
                    )
                }
                Query::Binary(op, lhs, rhs) => {
                    #[cfg(feature = "rayon")]
                    let (lhs_res, rhs_res) = rayon::join(
                        || run_internal_flat(*lhs, token),
                        || run_internal_flat(*rhs, token),
                    );

                    #[cfg(not(feature = "rayon"))]
                    let (lhs_res, rhs_res) = (
                        run_internal_flat(*lhs, token),
                        run_internal_flat(*rhs, token),
                    );

                    let mut lhs_res = lhs_res?.into_iter().collect::<WorkingTree<V>>();
                    let rhs_res = rhs_res?.into_iter().collect::<WorkingTree<V>>();
                    op.run(&mut lhs_res, &rhs_res)?;
                    Ok(lhs_res.into_iter().collect())
                }
                Query::Error(e) => Err(e),
            }
        }
        let resolver = match &self {
            Query::Unary(ops, _) | Query::CommutativeGroup(_, ops, _) => {
                ops.iter().rev().find_map(|op| op.tree_resolver())
            }
            _ => None,
        };
        let items = run_internal_flat(self, &CancellationToken::never())?;
        let tree = if let Some(resolve_fn) = resolver {
            WorkingTree::from_core(
                crate::spatial_id::collection::flex_tree::core::FlexTreeCore::par_build_vec_with(
                    items, resolve_fn,
                ),
            )
        } else {
            items.into_iter().collect()
        };
        Ok(tree)
    }
}

// Queryの遅延実行
impl<V: SafeValue + 'static> Query<V> {
    /// 出力領域 `bounds` を得るのに必要な入力領域を逆算しながら、その部分だけを評価する。
    ///
    /// `token` がキャンセルされると、AST を辿る途中で気づき次第 [`Error::Cancelled`] を返す。
    /// 出力領域 `bounds` を得るのに必要な入力領域を逆算しながら、その部分だけを評価する。
    ///
    /// `token` がキャンセルされると、AST を辿る途中で気づき次第 [`Error::Cancelled`] を返す。
    pub fn run_within(
        &self,
        bounds: Vec<crate::RangeId>,
        token: &CancellationToken,
    ) -> Result<WorkingTree<V>, Error> {
        trace_span!(
            "kasane_logic.query.run_within",
            target_regions = bounds.len()
        );
        self.validate()?;
        let items = self.run_within_flat(bounds, token)?;
        Ok(items.into_iter().collect())
    }

    /// [`run_within`](Self::run_within) の本体（再帰部分）。木を作らずフラットな Vec を返す。
    fn run_within_flat(
        &self,
        bounds: Vec<crate::RangeId>,
        token: &CancellationToken,
    ) -> Result<Vec<(crate::FlexId, V)>, Error> {
        if token.is_cancelled() {
            return Err(Error::Cancelled);
        }

        match self {
            Query::Source(s) => {
                trace_span!("kasane_logic.query.source_read", bound_count = bounds.len());
                s.read_range_ids_flat(&bounds, token)
            }
            Query::Unary(ops, input) | Query::CommutativeGroup(_, ops, input) => {
                trace_span!("kasane_logic.query.unary", op_count = ops.len());

                // 逆算は AST に書かれた順（実行の逆順）で辿る。並べ替えは可換な区間の
                // 中でしか起きず、可換な演算子同士は必要入力領域も入れ替わらない。
                let mut req = bounds;
                for op in ops.iter().rev() {
                    let mut next = Vec::new();
                    for r in req {
                        if let Some(inv) = op.inverse_bounds(r) {
                            next.push(inv);
                        }
                    }
                    next.sort_unstable();
                    next.dedup();
                    req = next;
                }
                let input_items = input.run_within_flat(req, token)?;
                {
                    trace_span!("kasane_logic.query.unary.apply");
                    composed_chain::run_composed_chain_flat(
                        &ops.optimized_order(),
                        input_items,
                        token,
                    )
                }
            }
            Query::Binary(op, lhs, rhs) => {
                trace_span!(
                    "kasane_logic.query.binary",
                    op = %core::fmt::from_fn(|f| op.fmt_op(f)),
                );

                let mut lhs_bounds = Vec::new();
                let mut rhs_bounds = Vec::new();
                for b in bounds {
                    let (l, r) = op.inverse_bounds(b.clone());
                    if let Some(lb) = l {
                        lhs_bounds.push(lb);
                    }
                    if let Some(rb) = r {
                        rhs_bounds.push(rb);
                    }
                }
                lhs_bounds.sort_unstable();
                lhs_bounds.dedup();
                rhs_bounds.sort_unstable();
                rhs_bounds.dedup();
                // 二項演算子は現在 WorkingTree を要求するため、ここで一度木を組む
                let lhs_items = lhs.run_within_flat(lhs_bounds, token)?;
                let mut lhs_working = lhs_items.into_iter().collect::<WorkingTree<V>>();
                let rhs_items = rhs.run_within_flat(rhs_bounds, token)?;
                let rhs_working = rhs_items.into_iter().collect::<WorkingTree<V>>();
                {
                    trace_span!(
                        "kasane_logic.query.binary.merge",
                        op = %core::fmt::from_fn(|f| op.fmt_op(f)),
                        lhs_count = lhs_working.count(),
                        rhs_count = rhs_working.count(),
                    );
                    op.run(&mut lhs_working, &rhs_working)?;
                }
                Ok(lhs_working.into_iter().collect())
            }
            Query::Error(e) => Err(e.clone()),
        }
    }

    /// 対象領域(`target`)と交差する部分だけを評価して返す。
    /// キャンセル不可。打ち切りたい場合は [`run_within`](Self::run_within) を使う。
    pub fn lazy_get<T: crate::SpatialId>(
        &self,
        target: T,
    ) -> Result<impl Iterator<Item = (crate::FlexId, V)>, Error> {
        let items =
            self.run_within_flat(vec![target.clone().into()], &CancellationToken::never())?;
        let target_range: crate::RangeId = target.into();

        Ok(items
            .into_iter()
            .filter(move |(id, _)| id.intersects_range(&target_range)))
    }

    /// 対象領域(`target`)のうち、データが存在しない空間を `default_value` で埋めてから値を返す。
    /// キャンセル不可。打ち切りたい場合は [`run_within`](Self::run_within) を使う。
    pub fn lazy_get_with_default<T: crate::SpatialId>(
        &self,
        target: T,
        default_value: V,
    ) -> Result<impl Iterator<Item = (crate::FlexId, V)>, Error> {
        let items =
            self.run_within_flat(vec![target.clone().into()], &CancellationToken::never())?;
        let target_range: crate::RangeId = target.clone().into();

        let mut uncovered = crate::SpatialIdSet::new();
        uncovered.insert(target.into());

        let mut working_iter = items.into_iter();
        let mut default_iter = None;

        Ok(core::iter::from_fn(move || {
            if default_iter.is_none() {
                for (id, value) in working_iter.by_ref() {
                    if id.intersects_range(&target_range) {
                        uncovered.remove(&id);
                        return Some((id, value));
                    }
                }
                default_iter = Some(core::mem::take(&mut uncovered).into_iter());
            }

            default_iter
                .as_mut()?
                .next()
                .map(|(id, _)| (id, default_value.clone()))
        }))
    }
}
