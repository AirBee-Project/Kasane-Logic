use crate::spatial_id::collection::flex_tree::core::{IntoIter, NoSummary};
use crate::spatial_id::collection::query::send_sync::{MaybeSend, MaybeSendSync, SafeValue};
use crate::{Error, FlexId, RangeId, SpatialIdTable};
use alloc::boxed::Box;
use alloc::vec::Vec;

#[cfg(not(feature = "rayon"))]
type BoxedIter<'a, V> = Box<dyn Iterator<Item = Result<(FlexId, V), Error>> + 'a>;
#[cfg(feature = "rayon")]
type BoxedIter<'a, V> = Box<dyn Iterator<Item = Result<(FlexId, V), Error>> + Send + 'a>;

enum QueryIterInner<'a, V> {
    Table(SpatialIdTable<V>),
    TableIter(IntoIter<V, NoSummary>),
    Stream(BoxedIter<'a, V>),
}

/// 演算子の間を流れる `(FlexId, V)` の列。カノニカル性を満たす。
pub struct QueryIter<'a, V>(QueryIterInner<'a, V>);

impl<'a, V> QueryIter<'a, V> {
    /// 制約を満たす `iter` を包む。制約を守るのは呼び出し側の責任。
    pub fn from_ordered(
        iter: impl Iterator<Item = Result<(FlexId, V), Error>> + MaybeSend + 'a,
    ) -> Self {
        #[cfg(debug_assertions)]
        let iter = {
            let mut prev: Option<FlexId> = None;
            iter.inspect(move |item| {
                if let Ok((id, _)) = item
                    && let Some(prev) = prev.replace(*id)
                {
                    assert_eq!(
                        prev.canonical_cmp(id),
                        Some(core::cmp::Ordering::Less),
                        "QueryIter の制約違反: {prev:?} の次に {id:?} が来た"
                    );
                }
            })
        };
        Self(QueryIterInner::Stream(Box::new(iter)))
    }

    /// 内部が未消費のテーブルであればそのまま取り出し、ストリームであれば収集してテーブルにする。
    pub(crate) fn into_table(self) -> Result<SpatialIdTable<V>, Error>
    where
        V: SafeValue,
    {
        match self.0 {
            QueryIterInner::Table(table) => Ok(table),
            QueryIterInner::TableIter(iter) => Ok(iter.collect()),
            QueryIterInner::Stream(iter) => iter.collect(),
        }
    }
}

impl<V: Clone> Iterator for QueryIter<'_, V> {
    type Item = Result<(FlexId, V), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            QueryIterInner::Table(table) => {
                let mut iter = core::mem::take(&mut table.inner).into_iter();
                let first = iter.next().map(Ok);
                self.0 = QueryIterInner::TableIter(iter);
                first
            }
            QueryIterInner::TableIter(iter) => iter.next().map(Ok),
            QueryIterInner::Stream(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.0 {
            QueryIterInner::Table(table) => {
                let n = table.inner.count();
                (n, Some(n))
            }
            QueryIterInner::TableIter(iter) => iter.size_hint(),
            QueryIterInner::Stream(iter) => iter.size_hint(),
        }
    }
}

impl<'a, V> From<SpatialIdTable<V>> for QueryIter<'a, V> {
    fn from(table: SpatialIdTable<V>) -> Self {
        Self(QueryIterInner::Table(table))
    }
}

/// 二項演算子の定義。
pub trait BinaryOperator<V: SafeValue + 'static>: MaybeSendSync {
    /// パラメーターの事前検証
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }

    /// [`QueryIter`] の制約を満たす `lhs` と `rhs` を受け取り、二項演算した結果を同じ制約を満たす列で返す。
    fn run<'a>(
        &'a self,
        lhs: QueryIter<'a, V>,
        rhs: QueryIter<'a, V>,
    ) -> Result<QueryIter<'a, V>, Error>;
}

/// 単項演算子の定義。
pub trait UnaryOperator<V: SafeValue + 'static>: MaybeSendSync {
    /// パラメーターの事前検証
    fn validate(&self) -> Result<(), Error>;

    /// [`QueryIter`] の制約を満たす `input` を受け取り、単項演算した結果を同じ制約を満たす列で返す。
    fn run<'a>(&'a self, input: QueryIter<'a, V>) -> Result<QueryIter<'a, V>, Error>;

    /// 与えられた出力領域を計算するために必要な入力領域を `out` へ追加する。
    ///
    /// `dyn UnaryOperator<V>` として動的ディスパッチするため `impl Iterator` は返せず、
    /// 呼び出しごとのヒープ確保を避けるため出力先ベクタを受け取る。
    fn reverse_target(&self, output_target: RangeId, out: &mut Vec<RangeId>);
}
