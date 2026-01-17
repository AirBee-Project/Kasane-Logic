use kasane_logic::spatial_id::{
    SpatialId,
    constants::{F_MAX, F_MIN, MAX_ZOOM_LEVEL, XY_MAX},
    range::RangeId,
    segment::{Segment, encode::EncodeSegment},
};
use rand::Rng;

fn main() {
    let id = RangeId::new(5, [-3, 10], [1, 4], [4, 5]).unwrap();

    println!("{}", id);

    let encode = id.encode();

    for ele in encode {
        println!("{},", ele.decode())
    }

    // let mut rng = rand::rng();
    // for i in 1..1000000 {
    //     let z = rng.random_range(0..MAX_ZOOM_LEVEL);
    //     let mut dim = [
    //         rng.random_range(F_MIN[z]..=F_MAX[z]),
    //         rng.random_range(F_MIN[z]..=F_MAX[z]),
    //     ];

    //     if dim[0] > dim[1] {
    //         dim.swap(0, 1);
    //     }

    //     for segment in Segment::<i32>::new(z as u8, dim) {
    //         let first = segment.clone();
    //         let encode = EncodeSegment::from(first);
    //         let decode = Segment::<i32>::from(encode.clone());

    //         if encode != decode.into() {
    //             println!("{:?}", first);
    //             println!("{}", encode);
    //             println!("{:?}", decode)
    //         }
    //     }
    // }
}
