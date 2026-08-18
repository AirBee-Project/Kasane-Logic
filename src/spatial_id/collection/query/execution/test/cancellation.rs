use alloc::vec;

use crate::spatial_id::collection::query::execution::run_unary_chain;
use crate::spatial_id::collection::query::traits::UnaryOperator;
use crate::{CancellationToken, Error, RangeId, SingleId, Source, SpatialIdTable};

#[test]
fn run_within_stops_immediately_when_already_cancelled() {
    let mut table = SpatialIdTable::<i32>::new();
    table.insert(SingleId::new(10, 0, 100, 100).unwrap(), 4);

    let token = CancellationToken::new();
    token.cancel();

    let bounds = vec![RangeId::from(&SingleId::new(10, 0, 100, 100).unwrap())];
    let result = table.query().shift_x(10, 1).run_within(bounds, &token);

    assert_eq!(result, Err(Error::Cancelled));
}

#[test]
fn run_within_completes_when_not_cancelled() {
    let mut table = SpatialIdTable::<i32>::new();
    table.insert(SingleId::new(10, 0, 100, 100).unwrap(), 4);

    let token = CancellationToken::new();
    let bounds = vec![RangeId::from(&SingleId::new(10, 0, 100, 100).unwrap())];

    let result = table.query().run_within(bounds, &token).unwrap();
    let values: alloc::vec::Vec<i32> = result.into_iter().map(|(_, v)| v).collect();

    assert_eq!(values, alloc::vec![4]);
}

#[test]
fn cancelling_after_one_side_still_stops_the_query() {
    let mut left = SpatialIdTable::<i32>::new();
    left.insert(SingleId::new(10, 0, 100, 100).unwrap(), 1);
    let mut right = SpatialIdTable::<i32>::new();
    right.insert(SingleId::new(10, 0, 200, 200).unwrap(), 2);

    let token = CancellationToken::new();
    token.cancel();

    let bounds = vec![
        RangeId::from(&SingleId::new(10, 0, 100, 100).unwrap()),
        RangeId::from(&SingleId::new(10, 0, 200, 200).unwrap()),
    ];
    let query = left.query().merge(
        right.query(),
        0,
        crate::spatial_id::collection::query::merge_policy::Sum,
    );

    assert_eq!(query.run_within(bounds, &token), Err(Error::Cancelled));
}

#[test]
fn run_unary_chain_checks_cancellation_between_ops() {
    let mut table = SpatialIdTable::<i32>::new();
    table.insert(SingleId::new(10, 0, 100, 100).unwrap(), 4);
    let working = table
        .read_range_ids(
            &[RangeId::from(&SingleId::new(10, 0, 100, 100).unwrap())],
            &CancellationToken::new(),
        )
        .unwrap();

    let a = crate::spatial_id::collection::query::ops::unary::shift::shift_x::ShiftX::new(10, 1)
        .unwrap();
    let b = crate::spatial_id::collection::query::ops::unary::shift::shift_x::ShiftX::new(10, 2)
        .unwrap();
    let ops: [&dyn UnaryOperator<i32>; 2] = [&a, &b];

    let token = CancellationToken::new();
    token.cancel();

    assert_eq!(
        run_unary_chain(&ops, working, &token),
        Err(Error::Cancelled)
    );
}

#[test]
fn never_cancelled_token_ignores_cancel() {
    let token = CancellationToken::never();
    token.cancel();
    assert!(!token.is_cancelled());
}
