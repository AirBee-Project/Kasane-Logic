use kasane_logic::spatial_id::segment::{Segment, encode::EncodeSegment};

fn main() {
    let test: Vec<Segment<i64>> = Segment::<i64>::new(2, [-1, -1]).collect();

    let first = test.first().unwrap();

    let encode = EncodeSegment::from(first.clone());

    println!("{:?}", first);
    println!("{}", encode);
    // println!("{:?}", Segment::<i64>::from(encode))
}
