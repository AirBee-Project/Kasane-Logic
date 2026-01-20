use rand::Rng;
use rayon::prelude::*;
use std::fs::File;
use std::io::Write;
use std::time::{Duration, Instant};

// クレート名は環境に合わせて修正してください
use kasane_logic::{
    SingleId, // RangeId から SingleId に変更
    spatial_id::{
        collection::{Collection, set::memory::SetOnMemory},
        constants::{F_MAX, F_MIN, XY_MAX},
    },
};

fn main() -> std::io::Result<()> {
    // 結果を書き込むファイル
    let mut file = File::create("benchmark_results_single.csv")?;
    writeln!(
        file,
        "count,insert_ms,remove_ms,union_ms,intersect_ms,difference_ms"
    )?;

    // 検証する要素数
    let steps: Vec<usize> = (100..=100000).step_by(100).collect();

    println!("Starting SingleId benchmark (Parallel execution)...");
    println!("Step   | Insert | Remove | Union  | Inter  | Diff");
    println!("-----------------------------------------------------");

    // Rayonによる並列ベンチマーク実行
    let mut results: Vec<(usize, f64, f64, f64, f64, f64)> = steps
        .par_iter()
        .map(|&count| run_benchmark_step(count))
        .collect();

    // 結果をソートして出力
    results.sort_by_key(|k| k.0);

    for (count, t_insert, t_remove, t_union, t_inter, t_diff) in results {
        println!(
            "{:>6} | {:>6.1} | {:>6.1} | {:>6.1} | {:>6.1} | {:>6.1}",
            count, t_insert, t_remove, t_union, t_inter, t_diff
        );

        writeln!(
            file,
            "{},{},{},{},{},{}",
            count, t_insert, t_remove, t_union, t_inter, t_diff
        )?;
    }

    println!("Benchmark finished. Results written to benchmark_results_single.csv");
    Ok(())
}

fn run_benchmark_step(count: usize) -> (usize, f64, f64, f64, f64, f64) {
    // ---------------------------------------------------------
    // 1. Insert & Remove Benchmark
    // ---------------------------------------------------------
    let mut set_main = SetOnMemory::default();

    // SingleIdを生成
    let ids_main = generate_mixed_zoom_single_ids_par(count);

    // --- Insert ---
    let start = Instant::now();
    for id in &ids_main {
        set_main.insert(id);
    }
    let t_insert = to_ms(start.elapsed());

    // --- Remove ---
    let mut set_for_remove = set_main.clone();
    let start = Instant::now();
    for id in &ids_main {
        set_for_remove.remove(id);
    }
    let t_remove = to_ms(start.elapsed());

    // ---------------------------------------------------------
    // 2. Set Operation Benchmark
    // ---------------------------------------------------------
    let mut set_a = SetOnMemory::default();
    let mut set_b = SetOnMemory::default();

    let ids_a = generate_mixed_zoom_single_ids_par(count);
    let ids_b = generate_mixed_zoom_single_ids_par(count);

    for id in &ids_a {
        set_a.insert(id);
    }
    for id in &ids_b {
        set_b.insert(id);
    }

    // --- Union ---
    let start = Instant::now();
    let _ = set_a.union(&set_b);
    let t_union = to_ms(start.elapsed());

    // --- Intersection ---
    // SingleIdの場合、ランダム生成だと重なりがほぼ発生しないため、
    // 処理時間は「スキャンして何も見つからない時間」になります。
    let start = Instant::now();
    let _ = set_a.intersection(&set_b);
    let t_inter = to_ms(start.elapsed());

    // --- Difference ---
    let start = Instant::now();
    let _ = set_a.difference(&set_b);
    let t_diff = to_ms(start.elapsed());

    (count, t_insert, t_remove, t_union, t_inter, t_diff)
}

fn to_ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

/// [Rayon版] ズームレベルが混在したSingleIdを生成する
fn generate_mixed_zoom_single_ids_par(count: usize) -> Vec<SingleId> {
    (0..count)
        .into_par_iter()
        .map_init(
            || rand::thread_rng(),
            |rng, _| {
                // Z=10 〜 Z=20 程度を混ぜる
                let z = rng.gen_range(10..=20);

                let f_min = F_MIN[z as usize];
                let f_max = F_MAX[z as usize];
                let xy_max = XY_MAX[z as usize];

                let f = rng.gen_range(f_min..=f_max);
                let x = rng.gen_range(0..=xy_max);
                let y = rng.gen_range(0..=xy_max);

                // SingleIdは点なので、有効な座標を指定すれば必ず生成できる（失敗しない）
                SingleId::new(z, f, x, y).expect("Invalid SingleId parameters generated")
            },
        )
        .collect()
}
