pub mod ast_optimization;
pub mod proptest_query;

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

/// 同じ(X,Y)・異なるFの2セルにextrude_fを適用すると、Max方針で全出力セルが最大値になる。
#[test]
fn extrude_f_same_xy_diff_f_resolves_via_policy() {
    let mut table = SpatialIdTable::new();
    table.insert(SingleId::new(10, 0, 100, 100).unwrap(), 10);
    table.insert(SingleId::new(10, 1, 100, 100).unwrap(), 20);

    let out = table
        .query()
        .extrude_f(
            10,
            -1,
            1,
            crate::spatial_id::collection::query::merge_policy::Max,
        )
        .raw_run()
        .unwrap();

    for (_, v) in out.flat_single_ids() {
        assert_eq!(*v, 20, "Max(10,20)は全出力セルで20になるはず");
    }
}

/// 回帰テスト: extrude_f/x/yは、列（他2軸のfootprint）でまとめて展開してから木を再構築する。
/// 列同士は「他軸で区別されていた粗いセルとその内側のネストした細かいセル」により
/// 互いに素とは限らないため、`from_flexids` へ渡す前の順序を決定的にしておかないと、
/// 同一入力でも実行のたびに結果が変わりうる（実データで実際に再現したバグ）。
#[test]
#[cfg(feature = "json")]
fn extrude_result_is_deterministic_across_runs() {
    let bldg_risk: SpatialIdTable<u32> =
        serde_json::from_str(&std::fs::read_to_string("sample/bldg_risk.json").unwrap()).unwrap();

    let mut subset = SpatialIdTable::new();
    for (id, &v) in bldg_risk.iter().take(200) {
        subset.insert(id.clone(), v);
    }

    let run_f = |t: SpatialIdTable<u32>| {
        t.query()
            .extrude_f(
                25,
                0,
                3,
                crate::spatial_id::collection::query::merge_policy::Max,
            )
            .raw_run()
            .unwrap()
    };
    let run_x = |t: SpatialIdTable<u32>| {
        t.query()
            .extrude_x(
                25,
                7450000,
                7450100,
                crate::spatial_id::collection::query::merge_policy::Max,
            )
            .raw_run()
            .unwrap()
    };
    let run_y = |t: SpatialIdTable<u32>| {
        t.query()
            .extrude_y(
                25,
                3301000,
                3301100,
                crate::spatial_id::collection::query::merge_policy::Max,
            )
            .raw_run()
            .unwrap()
    };

    for (label, run) in [
        (
            "extrude_f",
            &run_f as &dyn Fn(SpatialIdTable<u32>) -> SpatialIdTable<u32>,
        ),
        ("extrude_x", &run_x),
        ("extrude_y", &run_y),
    ] {
        let a = run(subset.clone());
        let b = run(subset.clone());
        assert_eq!(
            a.flat_single_ids().collect::<alloc::vec::Vec<_>>(),
            b.flat_single_ids().collect::<alloc::vec::Vec<_>>(),
            "{label}: 同一入力なのに結果が毎回変わる（非決定性あり）",
        );
    }
}
