use crate::spatial_id::collection::query::send_sync::SafeValue;
use crate::{Error, FlexId, SpatialIdTable, spatial_id::collection::query::traits::QueryIter};

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

/// `input` の各 [`FlexId`] を `shift` で移した先へ値を付けたまま移し、走査順に並べ直して返す。
///
/// `shift` が失敗した要素があれば、そのエラーを返す。
fn shift_all<'a, V, I>(
    input: QueryIter<'a, V>,
    shift: impl Fn(&FlexId) -> Result<I, Error>,
) -> Result<QueryIter<'a, V>, Error>
where
    V: SafeValue + 'a,
    I: Iterator<Item = FlexId>,
{
    // 移動の桁上がりで上位のビットが変わった要素は走査順の遠くへ跳ぶので、木に集めて並べ直す
    let mut moved = SpatialIdTable::new();
    for item in input {
        let (id, value) = item?;
        moved.extend(shift(&id)?.map(|id| (id, value.clone())));
    }
    Ok(moved.into())
}
