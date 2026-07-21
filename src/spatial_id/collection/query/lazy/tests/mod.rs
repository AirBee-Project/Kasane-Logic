use crate::{FlexId, SpatialIdCollection, SpatialIdTable};

#[test]
fn lazy_view_get_matches_run() {
    let mut table = SpatialIdTable::<u32>::new();
    let flex_id = FlexId::new(10, 10, 10, 10, 10, 10).unwrap();
    table.insert(flex_id.clone(), 42);

    // Normal run
    let expected_result = table
        .clone()
        .query()
        .shift_x(10, 1)
        .shift_y(10, 2)
        .raw_run()
        .unwrap();

    let target = FlexId::new(10, 10, 10, 11, 10, 12).unwrap();
    let expected_val = expected_result.get(&target).next().map(|(_, v)| *v);

    // LazyView get
    let query = table.query().shift_x(10, 1).shift_y(10, 2);
    let lazy_view = query.lazy();
    let lazy_val = lazy_view.get(target.clone()).unwrap();

    assert_eq!(1, lazy_val.len());
    assert_eq!(expected_val, Some(lazy_val[0].1));
}
