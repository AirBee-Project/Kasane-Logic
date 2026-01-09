use kasane_logic::spatial_id::segment::{Segment, encode::EncodeSegment};

fn main() {
    let test: Vec<Segment<u64>> = Segment::<u64>::new(3, [4, 4]).collect();

    let first = test.first().unwrap();

    let encode = EncodeSegment::from(first.clone());

    println!("{}", encode);
}
