use kasane_logic::{RangeId, SetOnMemory, SingleId, TableOnMemory};

fn main() {
    let mut table = TableOnMemory::default();
    let id = SingleId::new(3, 4, 5, 6).unwrap();
    table.insert(&id, &"neko".to_string());
    table.insert(&id, &"inu".to_string());

    println!("{}", table.to_set());
}
