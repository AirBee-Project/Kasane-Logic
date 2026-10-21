pub mod cancellation;
pub mod run;

use crate::{SingleId, Source, SpatialIdTable};

/// `run` はパラメータ検証を実行前に行うため、無効なパラメータはエラーになる
/// （実データ変換より先に検出される）。
#[test]
fn run_surfaces_validation_error() {
    let table: SpatialIdTable<i32> = SpatialIdTable::new();
    // zoom 100 は範囲外なので shift_x の構築時点で Query::Error になる。
    let result = table.query().shift_x(100, 3).run(..).table();
    assert!(result.is_err());
}

/// 同じ(X,Y)・異なるFの2Segmentにextrude_fを適用すると、Max方針で全出力Segmentが最大値になる。
#[test]
fn extrude_f_same_xy_diff_f_resolves_via_policy() {
    let mut table = SpatialIdTable::new();
    table.insert(SingleId::new(10, 0, 100, 100).unwrap(), 10);
    table.insert(SingleId::new(10, 1, 100, 100).unwrap(), 20);

    let out: SpatialIdTable<i32> = table
        .query()
        .extrude_f(
            10,
            -1,
            1,
            crate::spatial_id::collection::query::merge_policy::Max,
        )
        .run(..)
        .table()
        .unwrap();

    for (_, v) in out.flat_single_ids(None) {
        assert_eq!(*v, 20, "Max(10,20)は全出力Segmentで20になるはず");
    }
}

/// 回帰テスト: extrude_f/x/yは、列（他2軸のfootprint）でまとめて展開してから木を再構築する。
/// 列同士は「他軸で区別されていた粗いSegmentとその内側のネストした細かいSegment」により
/// 互いに素とは限らないため、`from_flexids` へ渡す前の順序を決定的にしておかないと、
/// 同一入力でも実行のたびに結果が変わりうる（実データで実際に再現したバグ）。
#[test]
#[cfg(feature = "json")]
fn extrude_result_is_deterministic_across_runs() {
    let bldg_risk: SpatialIdTable<u32> =
        serde_json::from_str(&std::fs::read_to_string("sample/bldg_risk.json").unwrap()).unwrap();

    let mut subset = SpatialIdTable::new();
    for (id, &v) in bldg_risk.iter().take(200) {
        subset.insert(id, v);
    }

    let run_f = |t: SpatialIdTable<u32>| {
        t.query()
            .extrude_f(
                25,
                0,
                3,
                crate::spatial_id::collection::query::merge_policy::Max,
            )
            .run(..)
            .table()
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
            .run(..)
            .table()
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
            .run(..)
            .table()
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
            a.flat_single_ids(None).collect::<alloc::vec::Vec<_>>(),
            b.flat_single_ids(None).collect::<alloc::vec::Vec<_>>(),
            "{label}: 同一入力なのに結果が毎回変わる（非決定性あり）",
        );
    }
}
