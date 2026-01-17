use std::collections::btree_map::Range;

use kasane_logic::spatial_id::{
    SpatialId,
    constants::{F_MAX, F_MIN, MAX_ZOOM_LEVEL, XY_MAX},
    range::RangeId,
    segment::{Segment, encode::EncodeSegment},
};
use rand::Rng;

fn main() {
    use std::fs::File;
    use std::io::{BufWriter, Write};

    let file1 = File::create("output1.txt").unwrap();
    let mut writer1 = BufWriter::new(file1);

    let file2 = File::create("output2.txt").unwrap();
    let mut writer2 = BufWriter::new(file2);

    let mut id = RangeId::new(5, [-3, 10], [5, 1], [5, 9]).unwrap();

    for ele in id.to_single() {
        writeln!(writer1, "{},", ele).unwrap();
    }

    let encode = id.encode();

    for ele in encode {
        writeln!(writer2, "{},", ele.decode()).unwrap();
    }

    // let mut rng = rand::rng();
    // let z = rng.random_range(0..4);
    // let mut f = [
    //     rng.random_range(F_MIN[z]..=F_MAX[z]),
    //     rng.random_range(F_MIN[z]..=F_MAX[z]),
    // ];
    // if f[0] > f[1] {
    //     f.swap(0, 1);
    // };
    // let mut x = [
    //     rng.random_range(0..=XY_MAX[z]),
    //     rng.random_range(0..=XY_MAX[z]),
    // ];
    // if x[0] > x[1] {
    //     x.swap(0, 1);
    // };

    // let mut y = [
    //     rng.random_range(0..=XY_MAX[z]),
    //     rng.random_range(0..=XY_MAX[z]),
    // ];
    // if y[0] > y[1] {
    //     y.swap(0, 1);
    // };

    // let id = RangeId::new(z as u8, f, x, y).unwrap();

    // println!("{}", id);

    // let encode = id.encode();

    // for ele in encode {
    //     println!("{},", ele.decode())
    // }

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
