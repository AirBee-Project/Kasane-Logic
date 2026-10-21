use alloc::vec;

use crate::spatial_id::collection::query::execution::run_unary_chain;
use crate::spatial_id::collection::query::traits::UnaryOperator;
use crate::{CancellationToken, Error, RangeId, SingleId, Source, SpatialIdTable};

#[test]
fn get_cancellable_stops_immediately_when_already_cancelled() {
    let mut table = SpatialIdTable::<i32>::new();
    table.insert(SingleId::new(10, 0, 100, 100).unwrap(), 4);

    let token = CancellationToken::new();
    token.cancel();

    let bounds = vec![RangeId::from(&SingleId::new(10, 0, 100, 100).unwrap())];
    let result = table
        .query()
        .shift_x(10, 1)
        .get_cancellable(bounds, &token)
        .table();

    assert_eq!(result, Err(Error::Cancelled));
}

#[test]
fn get_cancellable_completes_when_not_cancelled() {
    let mut table = SpatialIdTable::<i32>::new();
    table.insert(SingleId::new(10, 0, 100, 100).unwrap(), 4);

    let token = CancellationToken::new();
    let bounds = vec![RangeId::from(&SingleId::new(10, 0, 100, 100).unwrap())];

    let result = table
        .query()
        .get_cancellable(bounds, &token)
        .table()
        .unwrap();
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

    assert_eq!(
        query.get_cancellable(bounds, &token).table(),
        Err(Error::Cancelled)
    );
}

#[test]
fn run_unary_chain_checks_cancellation_between_ops() {
    let mut table = SpatialIdTable::<i32>::new();
    table.insert(SingleId::new(10, 0, 100, 100).unwrap(), 4);
    let bounds = [RangeId::new(10, 0, 100, 100).unwrap()];
    let stream = table
        .read_flex_ids(&bounds, &CancellationToken::new())
        .unwrap();

    let a =
        crate::spatial_id::collection::query::ops::unary::shift::Shift::new(10, 0, 1, 0).unwrap();
    let b =
        crate::spatial_id::collection::query::ops::unary::shift::Shift::new(10, 0, 2, 0).unwrap();
    let ops: [&dyn UnaryOperator<i32>; 2] = [&a, &b];

    let token = CancellationToken::new();
    token.cancel();

    let res = run_unary_chain(&ops, stream, &token);
    assert_eq!(res.err(), Some(Error::Cancelled));
}

#[test]
fn never_cancelled_token_ignores_cancel() {
    let token = CancellationToken::never();
    token.cancel();
    assert!(!token.is_cancelled());
}

#[test]
fn check_amortized_only_checks_periodically() {
    let token = CancellationToken::new();
    token.cancel();

    let mut ctr = 0u32;
    for _ in 0..4095 {
        assert!(token.check_amortized(&mut ctr).is_ok());
    }
    assert_eq!(token.check_amortized(&mut ctr), Err(Error::Cancelled));
}
