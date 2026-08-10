use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use kasane_logic::Source;
use kasane_logic::merge_policy::Max;

#[path = "../utils.rs"]
mod utils;

fn bench_extrude(c: &mut Criterion) {
    let mut group = c.benchmark_group("Unary/Extrude");
    group.sample_size(10);

    let table = utils::get_full_data();
    let distances: [i32; 7] = [1, 3, 5, 7, 10, 12, 15];

    // 個別の次元で行う関数 (X)
    for &dist in &distances {
        group.bench_with_input(BenchmarkId::new("extrude_x", dist), &dist, |b, &d| {
            b.iter_batched(
                || table.clone(),
                |t| t.query().extrude_x(24, 0, d as u32, Max).raw_run().unwrap(),
                BatchSize::SmallInput,
            );
        });
    }

    // 個別の次元で行う関数 (Y)
    for &dist in &distances {
        group.bench_with_input(BenchmarkId::new("extrude_y", dist), &dist, |b, &d| {
            b.iter_batched(
                || table.clone(),
                |t| t.query().extrude_y(24, 0, d as u32, Max).raw_run().unwrap(),
                BatchSize::SmallInput,
            );
        });
    }

    // 個別の次元で行う関数 (F)
    for &dist in &distances {
        group.bench_with_input(BenchmarkId::new("extrude_f", dist), &dist, |b, &d| {
            b.iter_batched(
                || table.clone(),
                |t| t.query().extrude_f(24, 0, d, Max).raw_run().unwrap(),
                BatchSize::SmallInput,
            );
        });
    }

    // 全ての次元で行う関数
    for &dist in &distances {
        group.bench_with_input(BenchmarkId::new("extrude_all", dist), &dist, |b, &d| {
            b.iter_batched(
                || table.clone(),
                |t| {
                    t.query()
                        .extrude_x(24, 0, d as u32, Max)
                        .extrude_y(24, 0, d as u32, Max)
                        .extrude_f(24, 0, d, Max)
                        .raw_run()
                        .unwrap()
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

criterion_group!(benches, bench_extrude);
criterion_main!(benches);
