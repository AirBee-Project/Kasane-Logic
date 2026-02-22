use crate::{RangeId, SetOnMemory, SingleId, spatial_id::single_id};
use std::collections::HashSet;

#[cfg(any(test))]
use proptest::prelude::Strategy;

pub mod difference;
pub mod insert;
pub mod intersection;
pub mod union;

///粒度を合わせてSingleIdで比較するためのヘルパー関数
/// テスト以外では使用しないため、ここに定義
#[cfg(any(test))]
pub fn to_flat_set(set: &SetOnMemory, target_z: u8) -> HashSet<SingleId> {
    let mut result = HashSet::new();
    for single_id in set.single_ids() {
        let diff = target_z - single_id.z();
        let children: Vec<_> = single_id.children(diff).unwrap().collect();
        result.extend(children);
    }
    result
}

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

    // SingleId と RangeId を半々の確率で選ぶ
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
