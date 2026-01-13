use std::collections::HashSet;

use kasane_logic::spatial_id::{
    constants::{F_MAX, F_MIN},
    segment::{self, Segment, encode::EncodeSegment},
};
use rand::Rng;

fn main() {
    let segment: Vec<_> = Segment::<i32>::new(1, [-2, 1]).collect();

    println!("{:?}", segment);
}

// fn main() {
//     let mut rng = rand::rng();
//     #[derive(PartialEq, Eq, Hash)]
//     struct Kiroku {
//         z: u8,
//         dimension: [i32; 2],
//     }
//     let mut set = HashSet::new();

//     for i in 0..1000000 {
//         let z = rng.random_range(0..20) as usize;
//         let mut dimension = [
//             rng.random_range(F_MIN[z]..=F_MAX[z]),
//             rng.random_range(F_MIN[z]..=F_MAX[z]),
//         ];
//         // println!("z={}", z);
//         // println!("{:?}", dimension);

//         if dimension[0] > dimension[1] {
//             dimension.swap(0, 1);
//         }

//         let segment: Vec<_> = Segment::<i32>::new(z as u8, dimension).collect();

//         if segment.first().is_none() {
//             set.insert(Kiroku {
//                 z: z as u8,
//                 dimension,
//             });
//         }

//         // let first = segment.first().unwrap().clone();

//         // let encode = EncodeSegment::from(first);

//         // let decode = Segment::<i32>::from(encode.clone());

//         // if first != decode {
//         //     println!("{:?}", first);
//         //     println!("{:?}", encode);
//         //     println!("{:?}", decode);

//         //     panic!()
//         // }
//     }

//     for ele in set {
//         println!("Z={} F={:?},", ele.z, ele.dimension);
//     }
// }
