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

/// 借用のまま実行順を決める `optimized_unary_order` が、AST を組み替える
/// [`Query::optimize`] と**同じ順序**を出すこと。
///
/// ここがずれると、同じクエリでも [`Query::run`]（AST を消費して最適化できる）と
/// [`Query::run_within`] / [`Query::lazy_get`]（`&self` しか持てない）で演算子の
/// 適用順が変わる。結果は可換なので一致するが、実行コストだけが黙って変わってしまう。
#[test]
fn borrowed_order_matches_the_optimized_ast() {
    use crate::spatial_id::collection::query::execution::group_commutative::optimized_unary_order;
    use alloc::vec;
    use alloc::vec::Vec;

    // 拡大率（`expansion_ratio` = |start - end| + 1）がバラバラな可換な3つ。
    let build = || {
        let table: SpatialIdTable<i32> = SpatialIdTable::new();
        table
            .query()
            .extrude_f(10, 0, 20, Max) // 21
            .extrude_x(10, 0, 2, Max) //  3
            .extrude_y(10, 0, 10, Max) // 11
    };

    let Query::Unary(ops, _) = build() else {
        panic!("最適化前は Unary のはず");
    };
    let borrowed: Vec<f32> = optimized_unary_order(&ops)
        .iter()
        .map(|op| op.expansion_ratio())
        .collect();

    let Query::CommutativeGroup(_, grouped, _) = build().optimize() else {
        panic!("3つとも可換なので CommutativeGroup になるはず");
    };
    let from_ast: Vec<f32> = grouped.iter().map(|op| op.expansion_ratio()).collect();

    assert_eq!(borrowed, from_ast, "借用経路と AST 経路で適用順が違う");
    assert_eq!(
        borrowed,
        vec![3.0, 11.0, 21.0],
        "拡大率の小さい順になっていない"
    );
}
