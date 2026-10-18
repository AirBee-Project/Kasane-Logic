use super::traits::{BinaryOperator, QueryIter, UnaryOperator};
use crate::Error;
use crate::spatial_id::collection::flex_tree::core::NoSummary;
use crate::spatial_id::collection::query::cancellation::CancellationToken;
use crate::{RangeId, SpatialIdTable};

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
        self.run_within(vec![RangeId::WHOLE], &CancellationToken::never())
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

/// 入力源から `bounds` と重なる要素を読み出し、読みながら `token` のキャンセルを確かめる列を返す。
fn read_source<'a, V: SafeValue + 'static>(
    source: &'a dyn Source<Value = V>,
    bounds: &[RangeId],
    token: &CancellationToken,
) -> Result<QueryIter<'a, V>, Error> {
    let token = token.clone();
    let mut counter = 0u32;
    Ok(QueryIter::from_ordered(
        source.read_flex_ids(bounds, &token)?.map(move |item| {
            token.check_amortized(&mut counter)?;
            item
        }),
    ))
}

// Queryの遅延実行

impl<V: SafeValue + 'static> Query<V> {
    /// 出力領域 `bounds` を得るのに必要な入力領域を逆算しながら、その部分だけを評価する。
    ///
    /// `token` がキャンセルされると、AST を辿る途中で気づき次第 [`Error::Cancelled`] を返す。
    pub fn run_within(
        &self,
        bounds: Vec<RangeId>,
        token: &CancellationToken,
    ) -> Result<SpatialIdTable<V, NoSummary>, Error> {
        self.validate()?;
        self.stream_within(bounds, token)?.collect()
    }

    /// 検証を行わずに、出力領域 `bounds` を得るのに必要な部分だけを評価した列を返す。
    ///
    /// `token` がキャンセルされると、AST を辿る途中か列を読む途中で [`Error::Cancelled`] を返す。
    pub(crate) fn stream_within<'a>(
        &'a self,
        bounds: Vec<RangeId>,
        token: &CancellationToken,
    ) -> Result<QueryIter<'a, V>, Error> {
        if token.is_cancelled() {
            return Err(Error::Cancelled);
        }
        if bounds.is_empty() {
            return Ok(QueryIter::from_ordered(core::iter::empty()));
        }

        match self {
            Query::Source(s) => read_source(&**s, &bounds, token),
            Query::Unary(ops, input) => {
                let mut req = bounds;
                for op in ops.iter().rev() {
                    let mut next = Vec::with_capacity(req.len());
                    for r in req {
                        op.reverse_target(r, &mut next);
                    }
                    next.sort_unstable();
                    next.dedup();
                    req = next;
                    if req.is_empty() {
                        return Ok(QueryIter::from_ordered(core::iter::empty()));
                    }
                }
                let input_stream = input.stream_within(req, token)?;
                let order: Vec<&dyn UnaryOperator<V>> = ops.iter().map(|op| &**op).collect();
                run_unary_chain(&order, input_stream, token)
            }
            Query::Binary(op, lhs, rhs) => {
                let lhs_bounds = bounds.clone();
                #[cfg(feature = "rayon")]
                let (l, r) = rayon::join(
                    || lhs.stream_within(lhs_bounds, token),
                    || rhs.stream_within(bounds, token),
                );
                #[cfg(not(feature = "rayon"))]
                let (l, r) = (
                    lhs.stream_within(lhs_bounds, token),
                    rhs.stream_within(bounds, token),
                );
                op.run(l?, r?)
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
