use std::collections::HashSet;

use crate::{
    RangeId, SetLogic, SetOnMemory, SetStorage, SingleId, spatial_id::collection::Collection,
};

pub mod difference;
pub mod insert;
pub mod intersection;
pub mod union;

///SetAを生成する
pub fn set_a() -> SetOnMemory {
    let mut set = SetOnMemory::default();
    let id1 = RangeId::new(5, [-7, 11], [1, 5], [5, 30]).unwrap();
    set.insert(&id1);
    let id2 = RangeId::new(3, [2, 2], [1, 5], [2, 2]).unwrap();
    set.insert(&id2);
    set
}

///SetBを生成する
pub fn set_b() -> SetOnMemory {
    let mut set = SetOnMemory::default();
    let id1 = RangeId::new(4, [5, 4], [4, 5], [9, 10]).unwrap();
    set.insert(&id1);
    let id2 = SingleId::new(2, 2, 2, 2).unwrap();
    set.insert(&id2);
    set
}

///SetCを生成する
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

///粒度を合わせてSingleIdで比較するためのヘルパー関数
/// テスト以外では使用しないため、ここに定義
pub fn to_flat_set<S>(set: &SetLogic<S>, target_z: u8) -> HashSet<SingleId>
where
    S: SetStorage + Collection + Default,
{
    // target_z が set.max_z() より深い場合のみ差分を計算
    let depth = target_z.saturating_sub(set.max_z());
    set.flatten_deep(depth)
        .expect("Failed to flatten set")
        .collect()
}
