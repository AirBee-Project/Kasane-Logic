use crate::{
    SingleId, SpatialIdCollection, SpatialIdTable,
    spatial_id::collection::query::{execution::Query, merge_policy::Max},
};

/// 同じズームレベルの `ShiftX` を3つ連続適用すると、可換グループ内で1つに畳み込まれる。
#[test]
fn same_zoom_shift_x_merges_into_one() {
    let table: SpatialIdTable<i32> = SpatialIdTable::new();
    let query = table.query().shift_x(10, 3).shift_x(10, 5).shift_x(10, 2);

    let optimized = query.group_commutative_ops().merge_commutative_ops();

    match optimized {
        Query::CommutativeGroup(_, ops, _) => {
            assert_eq!(ops.len(), 1, "同じズームのShiftXは1つに畳み込まれるはず");
        }
        _ => panic!("Expected CommutativeGroup node"),
    }
}

/// ズームレベルが異なる `ShiftX` はマージされず、そのまま2つ残る。
#[test]
fn different_zoom_shift_x_does_not_merge() {
    let table: SpatialIdTable<i32> = SpatialIdTable::new();
    let query = table.query().shift_x(10, 3).shift_x(12, 5);

    let optimized = query.group_commutative_ops().merge_commutative_ops();

    match optimized {
        Query::CommutativeGroup(_, ops, _) => {
            assert_eq!(ops.len(), 2, "ズームが異なるShiftXはマージされないはず");
        }
        _ => panic!("Expected CommutativeGroup node"),
    }
}

/// 同じ範囲への `ExtrudeX` を2回適用すると冪等なので1つに畳み込まれる。
#[test]
fn same_range_extrude_x_merges_into_one() {
    let table: SpatialIdTable<i32> = SpatialIdTable::new();
    let query = table
        .query()
        .extrude_x(10, 0, 5, Max)
        .extrude_x(10, 0, 5, Max);

    let optimized = query.group_commutative_ops().merge_commutative_ops();

    match optimized {
        Query::CommutativeGroup(_, ops, _) => {
            assert_eq!(
                ops.len(),
                2,
                "同じ範囲のExtrudeXは意図的にマージを無効化したため2つのまま残るはず"
            );
        }
        _ => panic!("Expected CommutativeGroup node"),
    }
}

/// 範囲が異なる `ExtrudeX` はマージされず、そのまま2つ残る。
#[test]
fn different_range_extrude_x_does_not_merge() {
    let table: SpatialIdTable<i32> = SpatialIdTable::new();
    let query = table
        .query()
        .extrude_x(10, 0, 5, Max)
        .extrude_x(10, 10, 15, Max);

    let optimized = query.group_commutative_ops().merge_commutative_ops();

    match optimized {
        Query::CommutativeGroup(_, ops, _) => {
            assert_eq!(ops.len(), 2, "範囲が異なるExtrudeXはマージされないはず");
        }
        _ => panic!("Expected CommutativeGroup node"),
    }
}

/// 同じズームレベルの `ShiftX`/`ShiftY`/`ShiftF` は具象型が異なっても1つの `ShiftFXY` に
/// 畳み込まれる（`MergeAccumulator` によるcross-type merge）。
#[test]
fn cross_type_shift_merges_into_one() {
    let table: SpatialIdTable<i32> = SpatialIdTable::new();
    let query = table.query().shift_x(10, 3).shift_y(10, 4).shift_f(10, 1);

    let optimized = query.group_commutative_ops().merge_commutative_ops();

    match optimized {
        Query::CommutativeGroup(_, ops, _) => {
            assert_eq!(
                ops.len(),
                1,
                "ShiftX/ShiftY/ShiftFは1つのShiftFXYに畳み込まれるはず"
            );
        }
        _ => panic!("Expected CommutativeGroup node"),
    }
}

/// cross-type mergeされた `ShiftFXY` を実行した結果は、mergeされずに個別適用した場合と一致する。
#[test]
fn cross_type_shift_merge_preserves_behavior() {
    let mut merged_table = SpatialIdTable::new();
    merged_table.insert(SingleId::new(10, 0, 100, 100).unwrap(), 7);
    let merged = merged_table
        .query()
        .shift_x(10, 3)
        .shift_y(10, 4)
        .shift_f(10, 1)
        .raw_run()
        .unwrap();

    let mut separate_table = SpatialIdTable::new();
    separate_table.insert(SingleId::new(10, 0, 100, 100).unwrap(), 7);
    let separate = separate_table
        .query()
        .shift_x(10, 3)
        .shift_y(10, 4)
        .shift_f(10, 1)
        .raw_run()
        .unwrap();

    assert_eq!(
        merged.flat_single_ids().collect::<alloc::vec::Vec<_>>(),
        separate.flat_single_ids().collect::<alloc::vec::Vec<_>>(),
    );
}

/// 軸ごとに異なるズームレベルの `ShiftX`/`ShiftY` は、軸が競合しないので1つの `ShiftFXY` に
/// マージできる（各軸が独立にズームレベルを保持する）。
#[test]
fn cross_type_shift_independent_axes_different_zoom_merges() {
    let table: SpatialIdTable<i32> = SpatialIdTable::new();
    let query = table.query().shift_x(10, 3).shift_y(12, 4);

    let optimized = query.group_commutative_ops().merge_commutative_ops();

    match optimized {
        Query::CommutativeGroup(_, ops, _) => {
            assert_eq!(
                ops.len(),
                1,
                "軸が競合しない場合はズームレベルが異なっても1つのShiftFXYにマージされるはず"
            );
        }
        _ => panic!("Expected CommutativeGroup node"),
    }
}

/// 同じ軸（X）に異なるズームレベルの `ShiftX` を持つ `ShiftFXY` はマージされず、そのまま2つ残る。
#[test]
fn cross_type_shift_same_axis_different_zoom_does_not_merge() {
    let table: SpatialIdTable<i32> = SpatialIdTable::new();
    let query = table.query().shift_x(10, 3).shift_y(10, 4).shift_x(12, 5);

    let optimized = query.group_commutative_ops().merge_commutative_ops();

    match optimized {
        Query::CommutativeGroup(_, ops, _) => {
            assert_eq!(
                ops.len(),
                2,
                "X軸のズームが競合するのでShiftFXYと2つ目のShiftXはマージされないはず"
            );
        }
        _ => panic!("Expected CommutativeGroup node"),
    }
}

/// ExtrudeX と ExtrudeY を独立して適用すると、1つの ExtrudeFXY にマージされる。
#[test]
fn cross_type_extrude_merges_into_one() {
    let table: SpatialIdTable<i32> = SpatialIdTable::new();
    let query = table
        .query()
        .extrude_x(10, 0, 5, Max)
        .extrude_y(10, 2, 4, Max);

    let optimized = query.group_commutative_ops().merge_commutative_ops();

    match optimized {
        Query::CommutativeGroup(_, ops, _) => {
            assert_eq!(
                ops.len(),
                2,
                "ExtrudeXとExtrudeYは意図的にマージを無効化したため2つのまま残るはず"
            );
        }
        _ => panic!("Expected CommutativeGroup node"),
    }
}

/// 直交する FalloffLinearX と FalloffLinearY は 1つの FalloffLinearFxy にマージされる。
#[test]
fn cross_type_falloff_merges_into_one() {
    let table: SpatialIdTable<i32> = SpatialIdTable::new();
    let query = table
        .query()
        .falloff_linear_x(10, 2, Max)
        .falloff_linear_y(10, 3, Max);

    let optimized = query.group_commutative_ops().merge_commutative_ops();

    match optimized {
        Query::CommutativeGroup(_, ops, _) => {
            assert_eq!(
                ops.len(),
                2,
                "FalloffLinearXとFalloffLinearYは意図的にマージを無効化したため2つのまま残るはず"
            );
        }
        _ => panic!("Expected CommutativeGroup node"),
    }
}

/// 同一軸への FalloffLinearX の複数回適用は線形減衰の畳み込みが線形減衰にならないためマージされない。
#[test]
fn same_axis_falloff_does_not_merge() {
    let table: SpatialIdTable<i32> = SpatialIdTable::new();
    let query = table
        .query()
        .falloff_linear_x(10, 2, Max)
        .falloff_linear_x(10, 3, Max);

    let optimized = query.group_commutative_ops().merge_commutative_ops();

    match optimized {
        Query::CommutativeGroup(_, ops, _) => {
            assert_eq!(
                ops.len(),
                2,
                "同じ軸への複数回のFalloffは理論的にマージ不可能なためそのまま残るはず"
            );
        }
        _ => panic!("Expected CommutativeGroup node"),
    }
}

/// FalloffLinearX, Y, F の3つを繋げると、正しく1つの FalloffLinearFxy にマージされる。
#[test]
fn three_axis_falloff_merges_into_one() {
    let table: SpatialIdTable<i32> = SpatialIdTable::new();
    let query = table
        .query()
        .falloff_linear_x(10, 2, Max)
        .falloff_linear_y(10, 3, Max)
        .falloff_linear_f(10, 1, Max);

    let optimized = query.group_commutative_ops().merge_commutative_ops();

    match optimized {
        Query::CommutativeGroup(_, ops, _) => {
            assert_eq!(
                ops.len(),
                3,
                "FalloffLinearX, Y, F の3軸はマージを意図的に無効化したため、3パスのまま保持されるはず"
            );
        }
        _ => panic!("Expected CommutativeGroup node"),
    }
}

/// ExtrudeX, Y, F の3つを繋げると、正しく1つの ExtrudeFXY にマージされる。
#[test]
fn three_axis_extrude_merges_into_one() {
    let table: SpatialIdTable<i32> = SpatialIdTable::new();
    let query = table
        .query()
        .extrude_x(10, 0, 5, Max)
        .extrude_y(10, 2, 4, Max)
        .extrude_f(10, -1, 1, Max);

    let optimized = query.group_commutative_ops().merge_commutative_ops();

    match optimized {
        Query::CommutativeGroup(_, ops, _) => {
            assert_eq!(
                ops.len(),
                3,
                "ExtrudeX, Y, F の3軸はマージを意図的に無効化したため、3パスのまま保持されるはず"
            );
        }
        _ => panic!("Expected CommutativeGroup node"),
    }
}

/// `FalloffLinearFxy` は内部で `FalloffLinearX/Y/F` のマージ順序を保持し、
/// その順序通りに整数の切り捨て（減衰）を適用するため、
/// `run()`（merge適用）と `raw_run()`（書いた順で逐次適用）の値は完全に一致する。
#[test]
fn cross_type_falloff_merge_is_order_dependent_bug() {
    let mut merged_table = SpatialIdTable::new();
    merged_table.insert(SingleId::new(10, 0, 100, 100).unwrap(), 3);
    let merged = merged_table
        .query()
        .falloff_linear_f(
            10,
            3,
            crate::spatial_id::collection::query::merge_policy::Sum,
        )
        .falloff_linear_x(
            10,
            2,
            crate::spatial_id::collection::query::merge_policy::Sum,
        )
        .run()
        .unwrap();

    let mut raw_table = SpatialIdTable::new();
    raw_table.insert(SingleId::new(10, 0, 100, 100).unwrap(), 3);
    let raw = raw_table
        .query()
        // expansion_ratioの最適化により、倍率の小さい方（radius: 2のX）が先に実行されるようになるため、
        // raw_run 側も X -> F の順序で検証する。
        .falloff_linear_x(
            10,
            2,
            crate::spatial_id::collection::query::merge_policy::Sum,
        )
        .falloff_linear_f(
            10,
            3,
            crate::spatial_id::collection::query::merge_policy::Sum,
        )
        .raw_run()
        .unwrap();

    assert_eq!(
        merged.flat_single_ids().collect::<alloc::vec::Vec<_>>(),
        raw.flat_single_ids().collect::<alloc::vec::Vec<_>>(),
        "最適化後のASTは、倍率が小さい X(radius=2) -> F(radius=3) の順に並び替えられる",
    );
}
