use std::collections::HashSet;

#[cfg(any(test))]
use crate::spatial_id::collection::set::SetOnMemory;
use crate::{RangeId, SingleId};
use proptest::prelude::Strategy;

pub mod difference;
pub mod insert;
pub mod intersection;
pub mod union;

///SetAを生成する
#[cfg(any(test))]
pub fn set_a() -> SetOnMemory {
    let mut set = SetOnMemory::default();
    let id1 = RangeId::new(5, [-7, 11], [1, 5], [5, 30]).unwrap();
    set.insert(&id1);
    let id2 = RangeId::new(3, [2, 2], [1, 5], [2, 2]).unwrap();
    set.insert(&id2);
    set
}

///SetBを生成する
#[cfg(any(test))]
pub fn set_b() -> SetOnMemory {
    let mut set = SetOnMemory::default();
    let id1 = RangeId::new(4, [5, 4], [4, 5], [9, 10]).unwrap();
    set.insert(&id1);
    let id2 = SingleId::new(2, 2, 2, 2).unwrap();
    set.insert(&id2);
    set
}

///SetCを生成する
#[cfg(any(test))]
pub fn set_c() -> SetOnMemory {
    let mut set = SetOnMemory::default();
    let id1 = SingleId::new(2, 1, 1, 1).unwrap();
    set.insert(&id1);
    let id2 = SingleId::new(3, 4, 4, 4).unwrap();
    set.insert(&id2);
    let id3 = RangeId::new(4, [-7, 11], [4, 10], [1, 9]).unwrap();
    set.insert(&id3);
    set
}

#[derive(Debug, Clone)]
enum TestElem {
    Single(SingleId),
    Range(RangeId),
}

///テストのために、ランダムなSetを生成する関数
/// 計算負荷の観点からズームレベルを4までに制限
#[cfg(any(test))]
pub fn arb_small_set(max_len: usize) -> impl Strategy<Value = SetOnMemory> {
    use proptest::prop_oneof;
    let z_range = 0u8..=4;

    // SingleId と RangeId を半々の確率で選ぶ戦略
    let elem_strategy = prop_oneof![
        SingleId::arb_within(z_range.clone()).prop_map(TestElem::Single),
        RangeId::arb_within(z_range).prop_map(TestElem::Range),
    ];

    proptest::collection::vec(elem_strategy, 0..=max_len).prop_map(|elems| {
        let mut set = SetOnMemory::default();
        for elem in elems {
            match elem {
                TestElem::Single(id) => set.insert(&id),
                TestElem::Range(id) => set.insert(&id),
            }
        }
        set
    })
}
