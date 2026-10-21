use crate::query::Query;
use crate::query::cancellation::CancellationToken;
use crate::query::query_iter::QueryIter;
use crate::query::send_sync::{MaybeSendSync, SafeValue};
use crate::{Error, RangeId};
use alloc::boxed::Box;
use alloc::vec::Vec;

/// クエリのデータを提供するTrait。[`SpatialIdMap`](crate::SpatialIdMap) やリモートのデータベースなどに実装する。
///
/// # 実装の仕方
///
/// [`read_flex_ids`](Self::read_flex_ids) は [`QueryIter`] の制約を満たす列を返す。
///
/// - FlexId の Z-order 列で読み出せるなら、[`QueryIter::from_ordered`] で包んで逐次に流す。
///   [`FlexId::zorder_key`](crate::FlexId::zorder_key) の昇順で保存したデータベースを、その順に一度だけ走査し、
///   `targets` のどれかと [`intersects_range`](crate::FlexId::intersects_range) する要素を返す場合がこれにあたる。
///   `targets` ごとに分けて読むと、複数の `targets` と重なる要素が重複し、順序も崩れうるので、
///   全ての `targets` をまとめて一度に読む。
pub trait Source: MaybeSendSync {
    type Value: SafeValue;

    /// `targets` のどれかと重なる要素を、それぞれ1回ずつ、[`QueryIter`] の制約を満たす列で返す。
    ///
    /// `targets` は互いに重なっていてもよく、順序も問わない。
    /// 返す要素は `targets` の外にはみ出していてもよい。
    fn read_flex_ids<'a>(
        &'a self,
        targets: &[RangeId],
        token: &CancellationToken,
    ) -> Result<QueryIter<'a, Self::Value>, Error>;

    fn query(self) -> Query<Self::Value>
    where
        Self: Sized + 'static,
    {
        Query::Source(Box::new(self))
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
