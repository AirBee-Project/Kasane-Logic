mod ast_optimization;

use crate::{SingleId, SpatialIdCollection, SpatialIdTable};

/// `run()`（最適化パイプライン込み）は `raw_run()`（無最適化）と同じ結果を返す。
/// merge対象（ShiftX/ShiftY/ShiftF）とmerge対象外（FalloffLinear）を混在させ、
/// 最適化の有無で挙動が変わらないことを確認する。
#[test]
fn run_matches_raw_run() {
    let mut optimized_table = SpatialIdTable::new();
    optimized_table.insert(SingleId::new(10, 0, 100, 100).unwrap(), 4);
    let optimized = optimized_table
        .query()
        .shift_x(10, 3)
        .shift_y(10, 4)
        .shift_f(10, 1)
        .falloff_linear_x(
            10,
            2,
            crate::spatial_id::collection::query::merge_policy::Sum,
        )
        .run()
        .unwrap();

    let mut raw_table = SpatialIdTable::new();
    raw_table.insert(SingleId::new(10, 0, 100, 100).unwrap(), 4);
    let raw = raw_table
        .query()
        .shift_x(10, 3)
        .shift_y(10, 4)
        .shift_f(10, 1)
        .falloff_linear_x(
            10,
            2,
            crate::spatial_id::collection::query::merge_policy::Sum,
        )
        .raw_run()
        .unwrap();

    assert_eq!(
        optimized.flat_single_ids().collect::<alloc::vec::Vec<_>>(),
        raw.flat_single_ids().collect::<alloc::vec::Vec<_>>(),
    );
}

/// `run()` はパラメータ検証を実行前に行うため、無効なパラメータはエラーになる
/// （最適化や実データ変換より先に検出される）。
#[test]
fn run_surfaces_validation_error() {
    let table: SpatialIdTable<i32> = SpatialIdTable::new();
    // zoom 100 は範囲外なので shift_x の構築時点で Query::Error になる。
    let result = table.query().shift_x(100, 3).run();
    assert!(result.is_err());
}
