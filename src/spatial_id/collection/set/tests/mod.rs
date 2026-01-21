use crate::{RangeId, SetOnMemory, SingleId};

pub mod equal;
pub mod insert;
pub mod intersection;
pub mod union;

///SetAを生成する
///SetAはBとは一切交わらない
/// SetCと交わる

pub fn set_a() -> SetOnMemory {
    let mut set = SetOnMemory::default();
    let id1 = RangeId::new(5, [-7, 11], [1, 5], [5, 30]).unwrap();
    set.insert(&id1);
    let id2 = RangeId::new(3, [2, 2], [1, 5], [2, 2]).unwrap();
    set.insert(&id2);
    set
}

///SetBを生成する
///SetBはAとは一切交わらない
/// SetCと交わる
pub fn set_b() -> SetOnMemory {
    let mut set = SetOnMemory::default();
    let id1 = RangeId::new(4, [5, 4], [4, 5], [9, 10]).unwrap();
    set.insert(&id1);
    let id2 = SingleId::new(2, 2, 2, 2).unwrap();
    set.insert(&id2);
    set
}

///SetCを生成する
///SetAとBと交わる
pub fn set_c() -> SetOnMemory {
    let mut set = SetOnMemory::default();
    let id1 = SingleId::new(2, 1, 1, 1).unwrap();
    set.insert(&id1);
    let id2 = SingleId::new(3, 4, 4, 4).unwrap();
    set.insert(&id2);
    set
}
