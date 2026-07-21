use crate::{
    SpatialIdCollection, SpatialIdTable, spatial_id::collection::query::execution::Query,
    spatial_id::collection::query::merge_policy::Max,
};

/// AST中に `Query::CommutativeGroup` ノードが1つでも存在するか（再帰探索）。
fn contains_commutative_group<S: SpatialIdCollection>(query: &Query<S>) -> bool
where
    S::Value: 'static,
{
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
