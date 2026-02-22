use kasane_logic::SingleId;
use kasane_logic::geometry::{point::coordinate::Coordinate, shapes::triangle};
use kasane_logic::triangle::Triangle;
use std::collections::HashSet;
#[cfg(feature = "random")]
use std::fmt::Error;
use std::time::Instant;
//use rand::prelude::*;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;

const MIN_LAT: f64 = 20.0;
const MAX_LAT: f64 = 46.0;
const MIN_LON: f64 = 122.0;
const MAX_LON: f64 = 154.0;
const MIN_ALT: f64 = 0.0;
const MAX_ALT: f64 = 1000.0;
#[cfg(feature = "random")]
pub fn generate_random_point() -> Result<Coordinate, kasane_logic::Error> {
    use rand::Rng; // 0.9系でもトレイトのインポートが必要

    let mut rng = rand::rng();
    Ok(Coordinate::new(
        rng.random_range(MIN_LAT..MAX_LAT),
        rng.random_range(MIN_LON..MAX_LON),
        rng.random_range(MIN_ALT..MAX_ALT),
    )?)
}

fn save_results_to_csv(z: u8, step: u32, file_path: &str) -> std::io::Result<()> {
    let (area, time, c1, c2) = resaerch(z, step).unwrap();
    // 1. 追記モードでファイルを開く（なければ作成）
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)?;

    // 2. ファイルが新規（サイズ0）ならヘッダーを書き込む
    if file.metadata()?.len() == 0 {
        writeln!(file, "length,time,c1,c2")?;
    }

    // 3. データをカンマ区切りで一行書き込む
    // 引数の l も含めて記録すると後で分析しやすくなります
    writeln!(file, "{},{},{},{}", area, time, c1, c2)?;

    // 4. フラッシュ（強制書き込み）
    file.flush()?;

    Ok(())
}

fn resaerch(z: u8, step: u32) -> Result<(f64, f64, usize, usize), Box<dyn std::error::Error>> {
    let mut file = File::create("output.txt")?;
    let tokyo = Coordinate::new(35.681382, 139.76608399999998, 0.0)?;
    let nagoya = Coordinate::new(35.1706431, 136.8816945, 100.0)?;
    let yokohama = Coordinate::new(35.4660694, 139.6226196, 100.0)?;
    let ikebukuro = Coordinate::new(35.728926, 139.71038, 100.0)?;
    let shinagawa = Coordinate::new(35.630152, 139.74044000000004, 800.0)?;
    let tri = Triangle::new([tokyo, ikebukuro, shinagawa]);
    let area = tri.area();
    let start_old = Instant::now();
    let set_old: HashSet<SingleId> = tri.single_ids(z)?.collect();
    let duration_old = start_old.elapsed();
    println!("従来関数実行時間: {:?}", duration_old);
    let start = Instant::now();
    let set: HashSet<SingleId> = tri.single_ids_neo(z, step)?.collect();
    let duration = start.elapsed();
    println!("新関数実行時間: {:?}", duration);
    let c1 = set.intersection(&set_old).count();
    let c2 = set_old.difference(&set).count();
    let c3 = set.difference(&set_old).count();
    println!("重複要素{},旧来のみ{},新のみ{}", c1, c2, c3,);
    for id in set {
        writeln!(file, "{},", id)?;
    }
    let speed = duration_old.as_secs_f64() / duration.as_secs_f64();
    println!("{}倍の高速化に成功", speed);
    println!("結果を output.txt に保存しました 。");
    Ok((area, speed, c2, c3))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
