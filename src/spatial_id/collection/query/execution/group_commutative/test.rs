use crate::{
    SpatialIdCollection, SpatialIdTable, spatial_id::collection::query::execution::Query,
    spatial_id::collection::query::merge_policy::Max,
};

#[test]
fn test_group_commutative_ops() {
    let table: SpatialIdTable<i32> = SpatialIdTable::new();
    // Shift is not currently mapped in commutativity_info (Unknown by default),
    // Extrude is explicitly mapped as Extrude (commutative).
    // Let's chain extrude_f, extrude_x, extrude_y using the SAME merge policy (Max).
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
