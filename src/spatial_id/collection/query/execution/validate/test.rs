use crate::{Source, SpatialIdTable, spatial_id::collection::query::merge_policy::KeepExisting};

#[test]
fn test_query_validate_success() {
    let table: SpatialIdTable<i32> = SpatialIdTable::new();
    let query = table.query().shift_f(10, 5);
    assert!(query.validate().is_ok());
}

#[test]
fn test_query_validate_error() {
    let table: SpatialIdTable<i32> = SpatialIdTable::new();
    let query = table.query().extrude_f(10, -999999, 5, KeepExisting);
    assert!(query.validate().is_err());
}
