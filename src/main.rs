use kasane_logic::{RangeId, SetOnMemory, SingleId};

fn main() {
    let set_a = set_a();
    let set_b = set_b();
    let set_c = set_c();

    println!("{}", set_a);
    println!("{}", set_b);
    println!("{}", set_c);

    let diff_ab = set_a.difference(&set_b);
    let logic_result = diff_ab.difference(&set_c);

    println!("{}", logic_result);
}

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
