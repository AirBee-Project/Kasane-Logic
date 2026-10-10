use super::traits::{BinaryOperator, UnaryOperator};
use crate::Error;
use crate::SpatialIdTable;
use crate::spatial_id::collection::flex_tree::core::{NoSummary, SafeValue};
use crate::spatial_id::collection::query::cancellation::CancellationToken;
use crate::spatial_id::collection::query::execution::group_commutative::runs::UnaryOperatorSliceExt;
use crate::spatial_id::collection::query::execution::group_commutative::types::CommutativityInfo;
use crate::spatial_id::collection::query::grid::try_run_grid;
use crate::spatial_id::collection::query::source::Source;
use crate::trace::trace_span;
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

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
    /// 検証・AST最適化を適用して実行し、[`SpatialIdTable`] として返す。
    ///
    /// 値で速く絞り込みたいときは、結果に [`with_summary`](SpatialIdTable::with_summary) で Summary を付ける。
    pub fn run(self) -> Result<SpatialIdTable<V>, Error> {
        self.validate()?;
        self.optimize().raw_run()
    }

    /// 検証も最適化もせず [`Query`] を実行し、[`SpatialIdTable`] として返す。
    ///
    /// AST を組み替えず書かれた順序のまま実行する。最適化の有無を比べるための口で、通常は [`run`](Self::run) を使う。
    pub fn raw_run(self) -> Result<SpatialIdTable<V>, Error> {
        fn run_internal<V: SafeValue + 'static>(
            query: Query<V>,
            token: &CancellationToken,
        ) -> Result<SpatialIdTable<V, NoSummary>, Error> {
            match query {
                Query::Source(source) => read_source(
                    &*source,
                    &[crate::FlexId::LOWER_MAX, crate::FlexId::UPPER_MAX],
                    token,
                ),
                Query::Unary(ops, input) | Query::CommutativeGroup(_, ops, input) => {
                    let order: Vec<&dyn UnaryOperator<V>> = ops.iter().map(|op| &**op).collect();
                    run_unary_chain(&order, run_internal(*input, token)?, token)
                }
                Query::Binary(op, lhs, rhs) => {
                    #[cfg(feature = "rayon")]
                    let (lhs_res, rhs_res) =
                        rayon::join(|| run_internal(*lhs, token), || run_internal(*rhs, token));

                    #[cfg(not(feature = "rayon"))]
                    let (lhs_res, rhs_res) = (run_internal(*lhs, token), run_internal(*rhs, token));

                    let mut lhs_res = lhs_res?;
                    let rhs_res = rhs_res?;
                    op.run(&mut lhs_res, &rhs_res)?;
                    Ok(lhs_res)
                }
                Query::Error(e) => Err(e),
            }
        }
        run_internal(self, &CancellationToken::never())
    }
}

/// 単項演算の並びを順に適用する。
pub(crate) fn run_unary_chain<V: SafeValue + 'static>(
    mut ops: &[&dyn UnaryOperator<V>],
    mut working: SpatialIdTable<V, NoSummary>,
    token: &CancellationToken,
) -> Result<SpatialIdTable<V, NoSummary>, Error> {
    while let Some(head) = ops.first() {
        if token.is_cancelled() {
            return Err(Error::Cancelled);
        }

        // グリッドで実行できる演算の最長区間を取る。
        let mut grid_len = 0;
        let mut max_z = None;
        for op in ops.iter() {
            if let Some(z) = op.grid_zoom() {
                grid_len += 1;
                max_z = Some(max_z.map_or(z, |m: crate::ZoomLevel| m.max(z)));
            } else {
                break;
            }
        }

        if grid_len > 0 {
            let grid_ops = &ops[..grid_len];
            let grid_result = {
                trace_span!("kasane_logic.query.unary.grid", op_count = grid_len);
                try_run_grid(
                    &working,
                    grid_ops,
                    max_z.unwrap(),
                    grid_budget(&working),
                    token,
                )
            };
            if let Some(result) = grid_result {
                working = result?;
                ops = &ops[grid_len..];
                continue;
            }
        }
        {
            trace_span!(
                "kasane_logic.query.unary.op",
                op = %core::fmt::from_fn(|f| head.fmt_op(f)),
            );
            head.run(&mut working)?;
        }
        ops = &ops[1..];
    }
    Ok(working)
}

/// 入力源を読み出して木に組む。
fn read_source<V: SafeValue + 'static>(
    source: &dyn Source<Value = V>,
    bounds: &[crate::FlexId],
    token: &CancellationToken,
) -> Result<SpatialIdTable<V, NoSummary>, Error> {
    let mut counter = 0u32;

    source
        .read_flex_ids(bounds, token)?
        .map(|item| {
            token.check_amortized(&mut counter)?;
            item
        })
        .collect()
}

/// 平坦化を許す件数の上限。
fn grid_budget<V: SafeValue>(working: &SpatialIdTable<V, NoSummary>) -> u64 {
    (working.count() as u64)
        .saturating_mul(64)
        .saturating_add(1 << 20)
}

// Queryの遅延実行
impl<V: SafeValue + 'static> Query<V> {
    /// 出力領域 `bounds` を得るのに必要な入力領域を逆算しながら、その部分だけを評価する。
    ///
    /// `token` がキャンセルされると、AST を辿る途中で気づき次第 [`Error::Cancelled`] を返す。
    pub fn run_within(
        &self,
        bounds: Vec<crate::RangeId>,
        token: &CancellationToken,
    ) -> Result<SpatialIdTable<V, NoSummary>, Error> {
        trace_span!(
            "kasane_logic.query.run_within",
            target_regions = bounds.len()
        );
        self.validate()?;
        self.run_within_unchecked(bounds, token)
    }

    /// [`run_within`](Self::run_within) の本体（再帰部分）。
    fn run_within_unchecked(
        &self,
        bounds: Vec<crate::RangeId>,
        token: &CancellationToken,
    ) -> Result<SpatialIdTable<V, NoSummary>, Error> {
        if token.is_cancelled() {
            return Err(Error::Cancelled);
        }

        match self {
            Query::Source(s) => {
                let mut bounds: Vec<crate::FlexId> = bounds.into_iter().flatten().collect();
                bounds.sort_unstable();
                bounds.dedup();
                trace_span!("kasane_logic.query.source_read", bound_count = bounds.len());
                read_source(&**s, &bounds, token)
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
                let input_working = input.run_within_unchecked(req, token)?;
                {
                    trace_span!("kasane_logic.query.unary.apply");
                    run_unary_chain(&ops.optimized_order(), input_working, token)
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
                let mut lhs_working = lhs.run_within_unchecked(lhs_bounds, token)?;
                let rhs_working = rhs.run_within_unchecked(rhs_bounds, token)?;
                {
                    trace_span!(
                        "kasane_logic.query.binary.merge",
                        op = %core::fmt::from_fn(|f| op.fmt_op(f)),
                        lhs_count = lhs_working.count(),
                        rhs_count = rhs_working.count(),
                    );
                    op.run(&mut lhs_working, &rhs_working)?;
                }
                Ok(lhs_working)
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
        let working = self.run_within(vec![target.clone().into()], &CancellationToken::never())?;
        let target_range: crate::RangeId = target.into();

        Ok(working
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
        let working = self.run_within(vec![target.clone().into()], &CancellationToken::never())?;
        let target_range: crate::RangeId = target.clone().into();

        let mut uncovered = crate::SpatialIdSet::new();
        uncovered.insert(target.into());

        let mut working_iter = working.into_iter();
        let mut default_iter = None;

        Ok(core::iter::from_fn(move || {
            if default_iter.is_none() {
                for (id, value) in working_iter.by_ref() {
                    if id.intersects_range(&target_range) {
                        let _ = uncovered.remove(id);
                        return Some((id, value));
                    }
                }
                default_iter = Some(core::mem::take(&mut uncovered).into_iter());
            }

            default_iter
                .as_mut()?
                .next()
                .map(|id| (id, default_value.clone()))
        }))
    }
}
