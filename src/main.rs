use kasane_logic::space_id::single::SingleID;

fn main() {
    let mut id = SingleID::new(9, 30, 90, 55).unwrap();

    println!("{},", id);

    for z in 0..*id.as_z() - 1 {
        id = id.parent(1).unwrap();
        println!("{},", id);
    }
}
