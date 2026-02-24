triangle関数のデバッグ
traingle.rsのimplにこれを書く
```rs
pub fn single_ids_sumpling(&self, z: u8) -> Result<impl Iterator<Item = SingleId>, Error> {
    if z > MAX_ZOOM_LEVEL as u8 {
        return Err(Error::ZOutOfRange { z });
    }

    let ecef_a: Ecef = self.points[0].into();
    let ecef_b: Ecef = self.points[1].into();
    let ecef_c: Ecef = self.points[2].into();

    let min_lat_rad = self.points[0]
        .as_latitude()
        .abs()
        .min(self.points[1].as_latitude().abs())
        .min(self.points[2].as_latitude().abs())
        .to_radians();

    let d = PI * WGS84_A * min_lat_rad.cos() * 2f64.powi(-2 - z as i32);

    let l1 = ((ecef_c.as_x() - ecef_b.as_x()).powi(2)
        + (ecef_c.as_y() - ecef_b.as_y()).powi(2)
        + (ecef_c.as_z() - ecef_b.as_z()).powi(2))
    .sqrt();
    let l2 = ((ecef_a.as_x() - ecef_c.as_x()).powi(2)
        + (ecef_a.as_y() - ecef_c.as_y()).powi(2)
        + (ecef_a.as_z() - ecef_c.as_z()).powi(2))
    .sqrt();
    let l3 = ((ecef_a.as_x() - ecef_b.as_x()).powi(2)
        + (ecef_a.as_y() - ecef_b.as_y()).powi(2)
        + (ecef_a.as_z() - ecef_b.as_z()).powi(2))
    .sqrt();

    let steps = (l1.max(l2).max(l3) / d).ceil() as usize;

    let seen = Rc::new(RefCell::new(HashSet::new()));

    let iter = (0..=steps).flat_map(move |i| {
        let t = i as f64 / steps as f64;

        let line1 = (
            ecef_a.as_x() * (1.0 - t) + ecef_b.as_x() * t,
            ecef_a.as_y() * (1.0 - t) + ecef_b.as_y() * t,
            ecef_a.as_z() * (1.0 - t) + ecef_b.as_z() * t,
        );
        let line2 = (
            ecef_a.as_x() * (1.0 - t) + ecef_c.as_x() * t,
            ecef_a.as_y() * (1.0 - t) + ecef_c.as_y() * t,
            ecef_a.as_z() * (1.0 - t) + ecef_c.as_z() * t,
        );

        let seen = seen.clone();

        (0..=i).filter_map(move |j| {
            let (x, y, z_pos) = if i == 0 {
                (ecef_a.as_x(), ecef_a.as_y(), ecef_a.as_z())
            } else {
                let s = j as f64 / i as f64;
                (
                    line1.0 * (1.0 - s) + line2.0 * s,
                    line1.1 * (1.0 - s) + line2.1 * s,
                    line1.2 * (1.0 - s) + line2.2 * s,
                )
            };

            if let Ok(voxel_id) = Ecef::new(x, y, z_pos).to_single_id(z) {
                let mut borrowed = seen.borrow_mut();
                if borrowed.insert(voxel_id.clone()) {
                    Some(voxel_id)
                } else {
                    None
                }
            } else {
                None
            }
        })
    });

    Ok(iter)
}
```

```rs
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
const MAX_LAT: f64 = 22.0;
const MIN_LON: f64 = 137.0;
const MAX_LON: f64 = 139.0;
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

fn save_results_to_csv(
    area: f64,
    step: u32,
    z: u8,
    time: f64,
    old: usize,
    new: usize,
    file_path: &str,
) -> std::io::Result<()> {
    // 1. 追記モードでファイルを開く（なければ作成）
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)?;

    // 2. ファイルが新規（サイズ0）ならヘッダーを書き込む
    if file.metadata()?.len() == 0 {
        writeln!(file, "area,step,z,time,old,new")?;
    }

    // 3. データをカンマ区切りで一行書き込む
    // 引数の l も含めて記録すると後で分析しやすくなります
    writeln!(file, "{},{},{},{},{},{}", area, step, z, time, old, new)?;

    // 4. フラッシュ（強制書き込み）
    file.flush()?;

    Ok(())
}

fn resaerch(z: u8, steps: Vec<u32>, tri: Triangle) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create("output.txt")?;
    let tokyo = Coordinate::new(35.681382, 139.76608399999998, 0.0)?;
    let nagoya = Coordinate::new(35.1706431, 136.8816945, 100.0)?;
    let yokohama = Coordinate::new(35.4660694, 139.6226196, 100.0)?;
    let ikebukuro = Coordinate::new(35.728926, 139.71038, 100.0)?;
    let shinagawa = Coordinate::new(35.630152, 139.74044000000004, 800.0)?;
    let area = tri.area();
    let start_old = Instant::now();
    let set_old: HashSet<SingleId> = tri.single_ids(z)?.collect();
    let duration_old = start_old.elapsed();
    println!("従来関数実行時間: {:?}", duration_old);
    for i in steps {
        let start = Instant::now();
        let set: HashSet<SingleId> = tri.single_ids_neo(z, i)?.collect();
        let duration = start.elapsed();
        println!("新関数実行時間: {:?}", duration);
        let c1 = set.intersection(&set_old).count();
        let c2 = set_old.difference(&set).count();
        let c3 = set.difference(&set_old).count();
        println!("重複要素{},旧来のみ{},新のみ{}", c1, c2, c3,);
        let speed = duration_old.as_secs_f64() / duration.as_secs_f64();
        println!("{}倍の高速化に成功", speed);
        save_results_to_csv(area, i, z, speed, c2, c3, "analyzer.csv");
        println!("{}", i)
    }
    Ok(())
}

fn print_ave_areas() -> Result<(), Box<dyn std::error::Error>> {
    let mut areas = 0.0;
    for i in 0..100 {
        let tri = Triangle::new([
            generate_random_point()?,
            generate_random_point()?,
            generate_random_point()?,
        ]);
        areas += tri.area()
    }
    print!("{}", areas);
    Ok(())
}

fn generate_area_tris(
    max_area: f64,
    min_area: f64,
    n: usize,
) -> Result<Vec<Triangle>, Box<dyn std::error::Error>> {
    let mut tris: Vec<Triangle> = Vec::new();
    while tris.len() <= n {
        let tri = Triangle::new([
            generate_random_point()?,
            generate_random_point()?,
            generate_random_point()?,
        ]);
        if tri.area() >= max_area && tri.area() <= min_area {
            tris.push(tri);
        }
    }
    Ok((tris))
}

fn main() {
    let max_steps = 100;
    let z = 22;
    let tokyo = Coordinate::new(35.681382, 139.76608399999998, 0.0).unwrap();
    let ikebukuro = Coordinate::new(35.728926, 139.71038, 100.0).unwrap();
    let shinagawa = Coordinate::new(35.630152, 139.74044000000004, 50.0).unwrap();
    let tri = Triangle::new([tokyo, ikebukuro, shinagawa]);
    let steps: Vec<u32> = (1..=max_steps).map(|k| k).collect();
    resaerch(z, steps, tri);
}
```