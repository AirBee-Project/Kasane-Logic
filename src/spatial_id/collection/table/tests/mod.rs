pub mod insert;

// use crate::{RangeId, SingleId};

// #[cfg(any(test))]
// use proptest::prelude::*;

// #[derive(Debug, Clone)]
// pub enum TestElem {
//     Single(SingleId),
//     Range(RangeId),
// }

// #[cfg(any(test))]
// pub fn arb_test_elems_zero_four() -> impl Strategy<Value = Vec<TestElem>> {
//     use proptest::prop_oneof;

//     // ズームレベルの範囲（0~4）
//     let z_range = 0u8..=4;

//     // SingleId か RangeId のどちらかを生成する戦略
//     let elem_strategy = prop_oneof![
//         SingleId::arb_within(z_range.clone()).prop_map(TestElem::Single),
//         RangeId::arb_within(z_range).prop_map(TestElem::Range),
//     ];

//     // 長さを「10個から100個」の範囲でランダム生成
//     proptest::collection::vec(elem_strategy, 10..=30)
// }

// #[cfg(any(test))]
// pub fn arb_test_elems_one_five() -> impl Strategy<Value = Vec<TestElem>> {
//     use proptest::prop_oneof;

//     // ズームレベルの範囲（0~4）
//     let z_range = 1..=5;

//     // SingleId か RangeId のどちらかを生成する戦略
//     let elem_strategy = prop_oneof![
//         SingleId::arb_within(z_range.clone()).prop_map(TestElem::Single),
//         RangeId::arb_within(z_range).prop_map(TestElem::Range),
//     ];

//     // 長さを「10個から100個」の範囲でランダム生成
//     proptest::collection::vec(elem_strategy, 10..=30)
// }
