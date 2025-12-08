use kasane_logic::space_id::single::SingleID;

fn main() {
    let id = SingleID::new(4, 3, 5, 4).unwrap();

    println!("{},", id);

    let children = id.children(2).unwrap().collect::<Vec<_>>();

    for ele in children {
        println!("{},", ele);
    }
}
