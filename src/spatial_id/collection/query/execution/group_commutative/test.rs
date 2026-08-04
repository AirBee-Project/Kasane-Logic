use crate::{
    Source, SpatialIdTable, spatial_id::collection::flex_tree::core::SafeValue,
    spatial_id::collection::query::execution::Query,
    spatial_id::collection::query::merge_policy::Max,
};

/// AST中に `Query::CommutativeGroup` ノードが1つでも存在するか（再帰探索）。
fn contains_commutative_group<V: SafeValue + 'static>(query: &Query<V>) -> bool {
    match query {
        Query::CommutativeGroup(..) => true,
        Query::Unary(_, input) => contains_commutative_group(input),
        Query::Binary(_, lhs, rhs) => {
            contains_commutative_group(lhs) || contains_commutative_group(rhs)
        }
        Query::Source(_) | Query::Error(_) => false,
    }
}

#[test]
fn test_group_commutative_ops() {
    let table: SpatialIdTable<i32> = SpatialIdTable::new();
    let query = table
        .query()
        .extrude_f(10, 0, 5, Max)
        .extrude_x(10, 0, 5, Max)
        .extrude_y(10, 0, 5, Max)
        .shift_f(10, 2); // Shift will break the chain (Unknown, not commutative)

    let grouped = query.group_commutative_ops();

    if let Query::Unary(ops, inner) = grouped {
        assert_eq!(ops.len(), 1, "Expected 1 shift operator at top level");

        if let Query::CommutativeGroup(_, comm_ops, _) = &*inner {
            assert_eq!(
                comm_ops.len(),
                3,
                "Expected 3 extrude operators in the commutative group"
            );
        } else {
            panic!("Expected CommutativeGroup node inside Unary");
        }
    } else {
        panic!("Expected Unary node at top level");
    }
}

/// `ExtrudeX<Max>` と `FalloffLinearX<Max>` は同じ `MergePolicy`（Max）を使うが、
/// 数式的なパターンが異なる（Extrudeは絶対座標へ写す変換でシフト同変ではなく、
/// FalloffLinearはソース相対距離のシフト同変カーネル変換）ため、可換グループとして
/// まとめてはならない（回帰テスト）。
#[test]
fn extrude_and_falloff_with_same_policy_do_not_group_together() {
    let table: SpatialIdTable<i32> = SpatialIdTable::new();
    let query = table
        .query()
        .extrude_x(10, 0, 5, Max)
        .falloff_linear_x(10, 2, Max);

    let grouped = query.group_commutative_ops();
    assert!(
        !contains_commutative_group(&grouped),
        "ExtrudeXとFalloffLinearXは可換グループにまとめられてはいけない"
    );
}

/// `plan_order`（`&self` のまま並び順だけを計算する版）が、所有権を消費する
/// `group_commutative_ops` + `sort_commutative_ops` と同じ判断をすることを確認する。
///
/// `falloff_linear_f(r=5)` は `expansion_ratio` が 11.0、`shift_x` / `shift_y` は
/// 既定の 1.0。3つとも作用する軸が異なる（F/X/Y）ので互いに可換（同じ軸のshiftと
/// falloffは範囲外でのエラー有無が変わるため不可換 —
/// `boundary_proptest::same_axis_shift_and_falloff_are_not_reordered` 参照）
/// なので1つのクリークにまとまり、比率の昇順（shift, shift, falloff）に並べ替わるはず。
#[test]
fn plan_order_sorts_a_commutative_clique_by_expansion_ratio() {
    let table: SpatialIdTable<i32> = SpatialIdTable::new();
    let query = table
        .query()
        .falloff_linear_f(10, 5, Max) // index 0, ratio 11.0
        .shift_x(10, 3) // index 1, ratio 1.0
        .shift_y(10, -2); // index 2, ratio 1.0

    if let Query::Unary(ops, _) = query {
        let order = super::plan_order(&ops);
        assert_eq!(
            order,
            alloc::vec![1, 2, 0],
            "expansion_ratioの昇順（shift, shift, falloff）に並ぶはず"
        );
    } else {
        panic!("Expected Unary node at top level");
    }
}

/// `plan_order` は可換でない演算子どうしの相対順序を変えない
/// （宣言順のまま素通りする）。
#[test]
fn plan_order_keeps_non_commutative_ops_in_declaration_order() {
    let table: SpatialIdTable<i32> = SpatialIdTable::new();
    let query = table
        .query()
        .zoom_out(
            5u8,
            crate::spatial_id::collection::query::merge_policy::Average,
        ) // index 0: Other
        .filter_eq(3); // index 1: Other

    if let Query::Unary(ops, _) = query {
        let order = super::plan_order(&ops);
        assert_eq!(order, alloc::vec![0, 1]);
    } else {
        panic!("Expected Unary node at top level");
    }
}
