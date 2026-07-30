use crate::{FlexId, RangeId, SingleId, spatial_id::time_cells::TimeCells};

/// [`SingleId`] を FlexTree のノードアドレス（[`FlexId`]）へ展開するイテレータ。
///
/// 空間部分は常にちょうど1セルなので、時間セルごとに「同じ空間セル＋その時間セル」を
/// 返すだけで済む。**ヒープ確保をしない**（`Box<dyn Iterator>` も中間 `Vec` も使わない）
/// のが要点で、挿入はこの経路を1件につき1回通るため固定費がそのまま効く。
///
/// 時間を使わない場合（および時間がちょうど1つの2進セルに収まる場合）は要素が1つだけになり、
/// 時間軸の無かった頃の `core::iter::once` と同じコストになる。
pub struct SingleIdCells {
    /// 空間部分（3軸とも `SingleId` のズーム）。時間だけを差し替えて使い回す。
    base: FlexId,
    cells: TimeCells,
}

impl Iterator for SingleIdCells {
    type Item = FlexId;

    fn next(&mut self) -> Option<FlexId> {
        let (zoom, index) = self.cells.next()?;
        Some(self.base.with_time_cell(zoom, index))
    }
}

/// [`SingleId`] を FlexTree のノードアドレス（[`FlexId`]）へ展開する。
///
/// 空間部分は常にちょうど1セルだが、時間部分は仕様通り任意の秒数の間隔を取れるため、2の冪秒の
/// セルへ分解すると複数個になりうる（例: 30分間隔なら最大5個）。そのため戻り値はイテレータで、
/// 全要素を合わせて元の [`SingleId`] の時空間領域をちょうど覆う。
impl IntoIterator for SingleId {
    type Item = FlexId;
    type IntoIter = SingleIdCells;

    fn into_iter(self) -> Self::IntoIter {
        SingleIdCells {
            // `SingleId` は3軸とも同じズームで、各インデックスは構築時に検証済み。
            base: unsafe {
                FlexId::new_unchecked(self.z(), self.f(), self.z(), self.x(), self.z(), self.y())
            },
            cells: self.time_cells(),
        }
    }
}

impl SingleId {
    pub fn single_ids(self) -> impl Iterator<Item = SingleId> {
        core::iter::once(self)
    }

    /// この [`SingleId`] を [`RangeId`] として見る（各次元が1点の範囲）。
    ///
    /// [`From`] 実装への薄い別名。`Box<dyn Iterator>` を返す [`RangeId::into_iter`] ではなく、
    /// 割り当てのない [`SingleId::into_iter`] を使いたい場面と取り違えないための目印でもある。
    pub fn to_range(&self) -> RangeId {
        RangeId::from(self)
    }
}
