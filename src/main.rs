use kasane_logic::RangeId;
use std::str::FromStr;

fn main() {
    let single_id = RangeId::from_str("5/10/10/10_10/10:9").unwrap();
    println!("{}", single_id);
}
