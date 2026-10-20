pub use super::traits::QueryIter;
use crate::spatial_id::collection::query::cancellation::CancellationToken;
use crate::spatial_id::collection::query::execution::Query;
use crate::spatial_id::collection::query::send_sync::MaybeSendSync;
use crate::spatial_id::collection::query::send_sync::SafeValue;
use crate::{Error, RangeId};
use alloc::boxed::Box;

/// クエリのデータを提供するTrait。[`SpatialIdTable`](crate::SpatialIdTable) やリモートのデータベースなどに実装する。
///
/// # 実装の仕方
///
/// [`read_flex_ids`](Self::read_flex_ids) は [`QueryIter`] の制約を満たす列を返す。
///
/// - Z-ordered sequence で読み出せるなら、[`QueryIter::from_ordered`] で包んで逐次に流す。
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

/// `Source` を実装する型を、二項演算子の引数などで直接 [`Query`] として渡せるようにする。
impl<V: SafeValue + 'static, S: Source<Value = V> + 'static> From<S> for Query<V> {
    fn from(source: S) -> Self {
        source.query()
    }
}
