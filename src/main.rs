use kasane_logic::Coordinate;

fn main() {
    let tokyo = Coordinate::new(35.681382, 139.76608399999998, 0.0).unwrap();

    println!("{}", tokyo.single_id(18).unwrap());
}
