use alloc::boxed::Box;

use crate::{FlexId, RangeId, SingleId};

/// [`SingleId`] を FlexTree のノードアドレス（[`FlexId`]）へ展開する。
///
/// 空間部分は常にちょうど1セルだが、時間部分は仕様通り任意の秒数の間隔を取れるため、2の冪秒の
/// セルへ分解すると複数個になりうる（例: 30分間隔なら最大5個）。そのため戻り値はイテレータで、
/// 全要素を合わせて元の [`SingleId`] の時空間領域をちょうど覆う。
impl IntoIterator for SingleId {
    type Item = FlexId;
    type IntoIter = Box<dyn Iterator<Item = FlexId>>;
    fn into_iter(self) -> Self::IntoIter {
        RangeId::from(self).into_iter()
    }
}

impl SingleId {
    pub fn single_ids(self) -> impl Iterator<Item = SingleId> {
        core::iter::once(self)
    }
}
