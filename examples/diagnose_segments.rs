use kasane_logic::{
    Side::Upper, Source, SpatialIdTable, merge_policy::Max,
    spatial_id::collection::query::cancellation::CancellationToken,
    spatial_id::collection::query::ops::unary::falloff::FalloffPattern,
};
use std::fs;
use std::time::Instant;

fn main() {
    let json_str = fs::read_to_string("sample/bldg_risk.json").unwrap();
    let table: SpatialIdTable<u32> = serde_json::from_str(&json_str).unwrap();

    println!(
        "raw segment count (table.iter().count()): {}",
        table.iter().count()
    );

    // risk_diffusionと同じチェーン
    let query = table
        .clone()
        .query()
        .zoom_out(22, Max)
        .falloff_f(25, 10, Some(Upper), FalloffPattern::Linear, Max)
        .falloff_x(25, 10, None, FalloffPattern::Linear, Max)
        .falloff_y(25, 10, None, FalloffPattern::Linear, Max);

    // run()自体は速いはずなので、まず全体の所要時間と出力件数を測る。
    let t0 = Instant::now();
    let full_count = query.run().unwrap().count();
    println!(
        "run() elapsed: {:?}, output count: {}",
        t0.elapsed(),
        full_count
    );

    // run_by_segments()の呼び出し自体(候補領域を先に全部集める部分)にかかる時間と、
    // その後の反復(1候補ごとにrun_withinする部分)にかかる時間を分けて測る。
    let t1 = Instant::now();
    let iter = query.run_by_segments(CancellationToken::never());
    println!(
        "run_by_segments() construction (candidate collection): {:?}",
        t1.elapsed()
    );

    let t2 = Instant::now();
    let mut processed = 0usize;
    for _ in iter {
        processed += 1;
        if t2.elapsed().as_secs_f64() > 3.0 {
            break;
        }
    }
    let elapsed = t2.elapsed();
    println!(
        "run_by_segments iteration: {} output items in {:?} ({:.1} items/sec) [aborted early for measurement]",
        processed,
        elapsed,
        processed as f64 / elapsed.as_secs_f64()
    );
}
