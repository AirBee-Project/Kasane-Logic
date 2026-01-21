use kasane_logic::{RangeId, SetOnMemory, SingleId};

fn main() {
    let b_and_c = set_b().intersection(&set_c());

    println!("{}", b_and_c);

    let mut answer = vec![SingleId::new(3, 2, 2, 2).unwrap()];

    let mut result: Vec<_> = b_and_c.single_ids().collect();

    println!("{:?}", result);
    println!("{:?}", answer);

    assert_eq!(answer.sort(), result.sort())
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
