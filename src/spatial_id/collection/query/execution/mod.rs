use super::Query;
use super::query_iter::QueryIter;
use super::traits::{IntoTargets, Source, UnaryOperator};
use crate::Error;
use crate::spatial_id::collection::query::cancellation::CancellationToken;
use crate::{FlexId, RangeId, SpatialIdTable};

use crate::spatial_id::collection::query::send_sync::SafeValue;

use alloc::vec::Vec;

#[cfg(test)]
mod test;

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

// Queryの評価
impl<V: SafeValue + 'static> Query<V> {
    /// `targets` と交差する要素を評価して返す。`..` を渡すと全体を評価する。
    ///
    /// 出力領域 `targets` を得るのに必要な入力領域を逆算し、その部分だけを評価する。
    /// 評価は最初の要素を読むときに始まり、読んだ分だけ進む。表にするには [`Run::table`] を使う。
    pub fn run(&self, targets: impl IntoTargets) -> Run<'_, V> {
        let targets = targets.into_targets();
        Run {
            query: self,
            whole: targets == [RangeId::WHOLE],
            targets,
            token: CancellationToken::never(),
            stream: None,
            finished: false,
        }
    }
}

/// [`Query::run`] の結果を順に返すイテレーター。
///
/// 検証や評価の開始に失敗すると、最初の要素として `Err` を返して終わる。
/// 読み出しや演算に失敗すると、その位置で `Err` を返す。
pub struct Run<'a, V: SafeValue + 'static> {
    query: &'a Query<V>,
    targets: Vec<RangeId>,
    // 全体を評価するときは、範囲との交差を確かめずに済ませる
    whole: bool,
    token: CancellationToken,
    stream: Option<QueryIter<'a, V>>,
    finished: bool,
}

impl<'a, V: SafeValue + 'static> Run<'a, V> {
    /// `token` がキャンセルされると、評価の途中で気づき次第 [`Error::Cancelled`] を返す。読み始める前に呼ぶ。
    pub fn cancel_with(mut self, token: &CancellationToken) -> Self {
        self.token = token.clone();
        self
    }

    /// 結果を [`SpatialIdTable`] にして返す。
    ///
    /// 値で速く絞り込みたいときは、結果に [`with_summary`](SpatialIdTable::with_summary) で Summary を付ける。
    pub fn table(self) -> Result<SpatialIdTable<V>, Error> {
        self.collect()
    }

    fn start(&self) -> Result<QueryIter<'a, V>, Error> {
        self.query.validate()?;
        self.query.stream_within(self.targets.clone(), &self.token)
    }
}

impl<V: SafeValue + 'static> Iterator for Run<'_, V> {
    type Item = Result<(FlexId, V), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stream.is_none() {
            if self.finished {
                return None;
            }
            match self.start() {
                Ok(stream) => self.stream = Some(stream),
                Err(e) => {
                    self.finished = true;
                    return Some(Err(e));
                }
            }
        }
        let (whole, targets) = (self.whole, &self.targets);
        // 入力源や演算は範囲の外にはみ出した要素も返しうるので、交差するものだけを残す
        self.stream.as_mut()?.find(|item| match item {
            Ok((id, _)) => whole || targets.iter().any(|t| id.intersects_range(t)),
            Err(_) => true,
        })
    }
}

/// 単項演算の並びを順に適用する。
pub(crate) fn run_unary_chain<'a, V: SafeValue + 'static>(
    ops: &'a [impl core::borrow::Borrow<dyn UnaryOperator<V>>],
    mut stream: QueryIter<'a, V>,
    token: &CancellationToken,
) -> Result<QueryIter<'a, V>, Error> {
    for op in ops {
        if token.is_cancelled() {
            return Err(Error::Cancelled);
        }

        stream = op.borrow().run(stream)?;
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

impl<V: SafeValue + 'static> Query<V> {
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
                run_unary_chain(ops, input_stream, token)
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
}
