use alloc::vec;

use crate::spatial_id::collection::query::cancellation::CancellationToken;
use crate::spatial_id::collection::query::traits::UnaryOperator;
use crate::{Error, FlexId, RangeId, SingleId, Source, SpatialIdTable, WorkingTree, ZoomLevel};

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
        crate::spatial_id::collection::query::execution::composed_chain::run_composed_chain(
            &ops, working, &token
        ),
        Err(Error::Cancelled)
    );
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

#[test]
fn try_run_grid_stops_when_cancelled() {
    use crate::spatial_id::collection::query::grid::try_run_grid;
    use crate::spatial_id::collection::query::ops::unary::shift::shift_x::ShiftX;

    let mut table = SpatialIdTable::<i32>::new();
    table.insert(SingleId::new(10, 0, 100, 100).unwrap(), 4);
    let working = table
        .read_range_ids(
            &[RangeId::from(&SingleId::new(10, 0, 100, 100).unwrap())],
            &CancellationToken::new(),
        )
        .unwrap();

    let op = ShiftX::new(10, 1).unwrap();
    let ops: [&dyn UnaryOperator<i32>; 1] = [&op];
    let max_z = <ShiftX as UnaryOperator<i32>>::grid_zoom(&op).unwrap();

    let token = CancellationToken::new();
    token.cancel();

    let result = try_run_grid(&working, &ops, max_z, u64::MAX, &token);
    assert!(matches!(result, Some(Err(Error::Cancelled))));
}

#[test]
fn from_tree_checks_cancellation_during_estimation() {
    use crate::spatial_id::collection::query::grid::UniformGrid;

    // check_amortized の間引き間隔(4096回)より多い葉数を用意し、from_tree 自身の
    // ループ内チェックが（try_run_grid の呼び出し前チェックに頼らず）機能することを確かめる。
    // 値を互い違いにして、隣接Segmentが同値統合されて葉数が縮まないようにする。
    let tree: WorkingTree<i32> = (0..5000u32)
        .map(|i| {
            (
                FlexId::new(12, 0, 12, i % 4000, 12, i / 4000).unwrap(),
                i as i32,
            )
        })
        .collect();

    let token = CancellationToken::new();
    token.cancel();

    let z = ZoomLevel::new(12).unwrap();
    let result = UniformGrid::from_tree(&tree, z, u64::MAX, &token);
    assert!(matches!(result, Some(Err(Error::Cancelled))));
}
