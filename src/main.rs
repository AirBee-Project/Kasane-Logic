use kasane_logic::SingleId;
use kasane_logic::geometry::{point::coordinate::Coordinate, shapes::triangle};
use kasane_logic::triangle::Triangle;
use std::collections::HashSet;
use std::time::Instant;
//use rand::prelude::*;
use std::fs::File;
use std::io::Write;

const MIN_LAT: f64 = 20.0;
const MAX_LAT: f64 = 46.0;
const MIN_LON: f64 = 122.0;
const MAX_LON: f64 = 154.0;
const MIN_ALT: f64 = 0.0;
const MAX_ALT: f64 = 1000.0;
// fn rondom_point(rng: &mut impl Rng) -> Coordinate {
//     Coordinate::new(
//         rng.random_range(MIN_LAT..MAX_LAT),
//         rng.random_range(MIN_LON..MAX_LON),
//         rng.random_range(MIN_ALT..MAX_ALT),
//     )
//     .unwrap()
// }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create("output.txt")?;
    let tokyo = Coordinate::new(35.681382, 139.76608399999998, 0.0)?;
    let nagoya = Coordinate::new(35.1706431, 136.8816945, 100.0)?;
    let yokohama = Coordinate::new(35.4660694, 139.6226196, 100.0)?;
    let ikebukuro = Coordinate::new(35.728926, 139.71038, 100.0)?;
    let shinagawa = Coordinate::new(35.630152, 139.74044000000004, 800.0)?;
    const Z: u8 = 20;
    let tri = Triangle::new([tokyo, ikebukuro, shinagawa]);
    let start_old = Instant::now();
    let set_old: HashSet<SingleId> = tri.single_ids(Z)?.collect();
    let duration_old = start_old.elapsed();
    println!("従来関数実行時間: {:?}", duration_old);
    let start = Instant::now();
    let set: HashSet<SingleId> = tri.single_ids_neo(Z)?.collect();
    let duration = start.elapsed();
    println!("新関数実行時間: {:?}", duration);
    //  println!("{},{}", start.to_id(z), goal.to_id(z));
    let c1 = set.intersection(&set_old).count();
    let c2 = set_old.difference(&set).count();
    let c3 = set.difference(&set_old).count();
    println!("重複要素{},旧来のみ{},新のみ{}", c1, c2, c3,);
    for id in set {
        writeln!(file, "{},", id)?;
    }
    println!(
        "{}倍の高速化に成功",
        duration_old.as_secs_f64() / duration.as_secs_f64()
    );
    Ok(println!("結果を output.txt に保存しました 。"))
}
