use super::traits::{BinaryOperator, QueryIter, UnaryOperator};
use crate::Error;
use crate::spatial_id::collection::flex_tree::core::NoSummary;
use crate::spatial_id::collection::query::cancellation::CancellationToken;
use crate::{SpatialIdSet, SpatialIdTable};

use crate::spatial_id::collection::query::send_sync::SafeValue;
use crate::spatial_id::collection::query::source::Source;

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

#[cfg(test)]
mod test;

/// Query全体を表現する型。
pub enum Query<V: SafeValue + 'static> {
    /// 演算の起点
    Source(Box<dyn Source<Value = V>>),

    /// 連続した単項演算子
    Unary(Vec<Box<dyn UnaryOperator<V>>>, Box<Query<V>>),

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

// Queryの検証
impl<V: SafeValue + 'static> Query<V> {
    /// 実行までに全てのQueryのパラメーターが正常値の範囲内か検証する。
    pub fn validate(&self) -> Result<(), Error> {
        match self {
            Query::Source(_) => Ok(()),
            Query::Unary(ops, input) => {
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
}

// Queryの全体実行
impl<V: SafeValue + 'static> Query<V> {
    /// 検証を行って実行し、[`SpatialIdTable`] として返す。
    ///
    /// 値で速く絞り込みたいときは、結果に [`with_summary`](SpatialIdTable::with_summary) で Summary を付ける。
    pub fn run(self) -> Result<SpatialIdTable<V>, Error> {
        self.validate()?;

        fn run_internal<V: SafeValue + 'static>(
            query: Query<V>,
            token: &CancellationToken,
        ) -> Result<SpatialIdTable<V, NoSummary>, Error> {
            match query {
                Query::Source(source) => read_source(&*source, &crate::FlexId::ROOTS, token),
                Query::Unary(ops, input) => {
                    let order: Vec<&dyn UnaryOperator<V>> = ops.iter().map(|op| &**op).collect();
                    let input_working = run_internal(*input, token)?;
                    let input_stream: QueryIter<'_, V> = input_working.into();
                    let out_stream = run_unary_chain(&order, input_stream, token)?;
                    out_stream.collect()
                }
                Query::Binary(op, lhs, rhs) => {
                    #[cfg(feature = "rayon")]
                    let (lhs_res, rhs_res) =
                        rayon::join(|| run_internal(*lhs, token), || run_internal(*rhs, token));

                    #[cfg(not(feature = "rayon"))]
                    let (lhs_res, rhs_res) = (run_internal(*lhs, token), run_internal(*rhs, token));

                    let lhs_res = lhs_res?;
                    let rhs_res = rhs_res?;
                    let lhs_stream: QueryIter<'_, V> = lhs_res.into();
                    let rhs_stream: QueryIter<'_, V> = rhs_res.into();
                    let out_stream = op.run(lhs_stream, rhs_stream)?;
                    out_stream.collect()
                }
                Query::Error(e) => Err(e),
            }
        }
        run_internal(self, &CancellationToken::never())
    }
}

/// 単項演算の並びを順に適用する。
pub(crate) fn run_unary_chain<'a, V: SafeValue + 'static>(
    mut ops: &[&dyn UnaryOperator<V>],
    mut stream: QueryIter<'a, V>,
    token: &CancellationToken,
) -> Result<QueryIter<'a, V>, Error> {
    while let Some(head) = ops.first() {
        if token.is_cancelled() {
            return Err(Error::Cancelled);
        }

        stream = head.run(stream)?;
        ops = &ops[1..];
    }
    Ok(stream)
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
                // 重なりや隣接をまとめ、入力源へのリクエストを小さくする
                let bounds: Vec<crate::FlexId> = bounds
                    .into_iter()
                    .collect::<SpatialIdSet>()
                    .into_iter()
                    .collect();
                read_source(&**s, &bounds, token)
            }
            Query::Unary(ops, input) => {
                let mut req = bounds;
                for op in ops.iter().rev() {
                    let mut next = Vec::new();
                    for r in req {
                        if let Some(inv) = op.reverse_target(r) {
                            next.push(inv);
                        }
                    }
                    next.sort_unstable();
                    next.dedup();
                    req = next;
                }
                let input_working = input.run_within_unchecked(req, token)?;
                let order: Vec<&dyn UnaryOperator<V>> = ops.iter().map(|op| &**op).collect();
                let input_stream: QueryIter<'_, V> = input_working.into();
                let out_stream = run_unary_chain(&order, input_stream, token)?;
                out_stream.collect()
            }
            Query::Binary(op, lhs, rhs) => {
                let mut lhs_bounds = Vec::new();
                let mut rhs_bounds = Vec::new();
                for b in bounds {
                    let (l, r) = op.reverse_target(b.clone());
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
                let lhs_working = lhs.run_within_unchecked(lhs_bounds, token)?;
                let rhs_working = rhs.run_within_unchecked(rhs_bounds, token)?;
                let lhs_stream: QueryIter<'_, V> = lhs_working.into();
                let rhs_stream: QueryIter<'_, V> = rhs_working.into();
                let out_stream = op.run(lhs_stream, rhs_stream)?;
                out_stream.collect()
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
