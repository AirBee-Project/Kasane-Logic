use crate::{Error, FlexId, spatial_id::collection::query::traits::QueryIter};
use alloc::boxed::Box;

/// X方向への移動演算子
pub mod shift_x;

/// Y方向への移動演算子
pub mod shift_y;

/// F方向への移動演算子
pub mod shift_f;

/// FlexId単体に対する実装
pub mod primitive;

/// Query型への実装
pub mod query;

/// `input` の各 [`FlexId`] を `shift` で移した先へ、値を付けたまま逐次に流す。
///
/// `shift` が失敗した要素は、その位置で `Err` として流す。
fn shift_stream<'a, V, I>(
    input: QueryIter<'a, V>,
    shift: impl Fn(&FlexId) -> Result<I, Error> + 'a,
) -> QueryIter<'a, V>
where
    V: Clone + 'a,
    I: Iterator<Item = FlexId> + 'a,
{
    Box::new(input.flat_map(move |item| {
        let (moved, err) = match item.and_then(|(id, value)| Ok((shift(&id)?, value))) {
            Ok((ids, value)) => (Some(ids.map(move |id| Ok((id, value.clone())))), None),
            Err(e) => (None, Some(Err(e))),
        };
        moved.into_iter().flatten().chain(err)
    }))
}
