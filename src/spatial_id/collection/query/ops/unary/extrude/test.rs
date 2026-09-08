use alloc::vec::Vec;

use crate::spatial_id::collection::query::cancellation::CancellationToken;
use crate::spatial_id::collection::query::merge_policy::Sum;
use crate::{FlexId, SingleId, Source, SpatialIdTable};

fn segment(x: u32, v: i32) -> (SingleId, i32) {
    (SingleId::new(20, 0, x, 0).unwrap(), v)
}

/// `run_by_segments`が`run()`と完全に同じ結果を返すこと。
///
/// extrudeはどのSegmentも同じ`[start,end]`へ写るため、`forward_bounds`は入力の位置に
/// 関係なく常に同じ領域を返す。異なる2つのSource Segmentから同じ領域へ2回`run_within`が
/// 呼ばれても、重複除去によって最終結果が二重にならないことを確認する。
#[test]
fn extrude_x_run_by_segments_matches_run() {
    let mut table = SpatialIdTable::new();
    table.insert(segment(100, 4).0, 4);
    table.insert(segment(300, 6).0, 6);

    let query = table.query().extrude_x(20, 50, 150, Sum);

    let mut full: Vec<(FlexId, i32)> = query.run().unwrap().collect();
    full.sort();

    let mut by_segments: Vec<(FlexId, i32)> = query
        .run_by_segments(CancellationToken::never())
        .map(|r| r.unwrap())
        .collect();
    by_segments.sort();

    assert_eq!(full, by_segments);
    assert!(!full.is_empty());
}
