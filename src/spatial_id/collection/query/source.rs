use crate::spatial_id::collection::query::cancellation::CancellationToken;
use crate::spatial_id::collection::query::execution::Query;
use crate::spatial_id::collection::query::send_sync::MaybeSendSync;
use crate::spatial_id::collection::query::send_sync::SafeValue;
use crate::{Error, FlexId};
use alloc::boxed::Box;

pub type SourceIter<'a, V> = Box<dyn Iterator<Item = Result<(FlexId, V), Error>> + 'a>;

/// クエリを実行するためのTrait。読み取りさえできればよい。
pub trait Source: MaybeSendSync {
    type Value: SafeValue;

    fn read_flex_ids<'a>(
        &'a self,
        bounds: &'a [FlexId],
        token: &CancellationToken,
    ) -> Result<SourceIter<'a, Self::Value>, Error>;

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
