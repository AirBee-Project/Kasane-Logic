use crate::spatial_id::collection::query::send_sync::{MaybeSend, MaybeSendSync, SafeValue};
use crate::{Error, FlexId, RangeId, SpatialIdTable};
use alloc::boxed::Box;
use alloc::vec::Vec;

/// 演算子の間を流れる `(FlexId, V)` の列。カノニカル性を満たす。
pub struct QueryIter<'a, V>(
    #[cfg(not(feature = "rayon"))] Box<dyn Iterator<Item = Result<(FlexId, V), Error>> + 'a>,
    #[cfg(feature = "rayon")] Box<dyn Iterator<Item = Result<(FlexId, V), Error>> + Send + 'a>,
);

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
        Self(Box::new(iter))
    }
}

impl<V> Iterator for QueryIter<'_, V> {
    type Item = Result<(FlexId, V), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, V: SafeValue + 'static, S: MaybeSendSync + 'static> From<SpatialIdTable<V, S>>
    for QueryIter<'a, V>
{
    fn from(table: SpatialIdTable<V, S>) -> Self {
        Self::from_ordered(table.into_iter().map(Ok))
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
    fn reverse_target(&self, output_target: RangeId, out: &mut Vec<RangeId>);
}
