use kasane_logic::{
    geometry::{
        coordinate::Coordinate,
        ecef::Ecef,
        shapes::line::{line, line_dda, line_new},
    },
    id::space_id::single::SingleID,
};
use rand::prelude::*;
use std::{collections::HashSet, fs::File};
use std::{fmt::Error, io::Write};

const MIN_LAT: f64 = 26.00; //35.6197;
const MAX_LAT: f64 = 35.7380;
const MIN_LON: f64 = 127.88; //139.68;
const MAX_LON: f64 = 139.7983;
const MIN_ALT: f64 = 0.0;
const MAX_ALT: f64 = 100.0;
fn rondom_point(rng: &mut impl Rng) -> Coordinate {
    Coordinate::new(
        rng.random_range(MIN_LAT..MAX_LAT),
        rng.random_range(MIN_LON..MAX_LON),
        rng.random_range(MIN_ALT..MAX_ALT),
    )
    .unwrap()
}
fn difference(a: Coordinate, b: Coordinate) -> f64 {
    let ecef_a: Ecef = a.into();
    let ecef_b: Ecef = b.into();
    let x = ecef_a.as_x() - ecef_b.as_x();
    let y = ecef_a.as_y() - ecef_b.as_y();
    let z = ecef_a.as_z() - ecef_b.as_z();
    (x * x + y * y + z * z).sqrt()
}
fn benchmark(z: u8, a: Coordinate, b: Coordinate) -> Result<(), Box<dyn std::error::Error>> {
    let old_line: HashSet<SingleID> = line(z, a, b)?.collect();
    let new_line: HashSet<SingleID> = line_new(z, a, b)?.collect();
    let common_count = old_line.intersection(&new_line).count();
    let only_in_old_count = old_line.difference(&new_line).count();
    let only_in_new_count = new_line.difference(&old_line).count();
    println!(
        "共通{},旧のみ{},新のみ{}",
        common_count, only_in_old_count, only_in_new_count
    );
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create("output.txt")?;
    let tokyo = Coordinate::new(35.681382, 139.76608399999998, 0.0)?;
    let nagoya = Coordinate::new(35.1706431, 136.8816945, 100.0)?;
    let yokohama = Coordinate::new(35.4660694, 139.6226196, 100.0)?;
    let (z, start, goal) = (17, tokyo, yokohama);
    // let iter = line_dda(z, start, goal)?;
    // for id in iter {
    //     // SingleIDの内容を一行ずつ書き込む
    //     writeln!(file, "{},", id)?;
    // }
    // println!("{},{}", start.to_id(z), goal.to_id(z));
    // Ok(println!("結果を output.txt に保存しました 。"))
    let mut my_rng = rand::rng();
    for _ in 0..10 {
        let a = rondom_point(&mut my_rng);
        let b = rondom_point(&mut my_rng);
        println!("距離{}m", difference(a, b));
        benchmark(z, a, b);
        println!("");
    }
    Ok(())
}
