use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use kasane_logic::Source;
use kasane_logic::merge_policy::Max;
use kasane_logic::spatial_id::collection::query::ops::unary::falloff::FalloffPattern;

#[path = "../utils.rs"]
mod utils;

fn bench_falloff(c: &mut Criterion) {
    let mut group = c.benchmark_group("Unary/Falloff");
    group.sample_size(10);

    let table = utils::get_full_data();
    let distances = [1, 3, 5, 7, 10, 12, 15];

    // 個別の次元で行う関数 (X)
    for &dist in &distances {
        group.bench_with_input(BenchmarkId::new("falloff_x", dist), &dist, |b, &d| {
            b.iter_batched(
                || table.clone(),
                |t| {
                    t.query()
                        .falloff_x(24, d as u32, None, FalloffPattern::Linear, Max)
                        .raw_run()
                        .unwrap()
                },
                BatchSize::SmallInput,
            );
        });
    }

    // 個別の次元で行う関数 (Y)
    for &dist in &distances {
        group.bench_with_input(BenchmarkId::new("falloff_y", dist), &dist, |b, &d| {
            b.iter_batched(
                || table.clone(),
                |t| {
                    t.query()
                        .falloff_y(24, d as u32, None, FalloffPattern::Linear, Max)
                        .raw_run()
                        .unwrap()
                },
                BatchSize::SmallInput,
            );
        });
    }

    // 個別の次元で行う関数 (F)
    for &dist in &distances {
        group.bench_with_input(BenchmarkId::new("falloff_f", dist), &dist, |b, &d| {
            b.iter_batched(
                || table.clone(),
                |t| {
                    t.query()
                        .falloff_f(24, d as u32, None, FalloffPattern::Linear, Max)
                        .raw_run()
                        .unwrap()
                },
                BatchSize::SmallInput,
            );
        });
    }

    // 全ての次元で行う関数
    for &dist in &distances {
        group.bench_with_input(BenchmarkId::new("falloff_all", dist), &dist, |b, &d| {
            b.iter_batched(
                || table.clone(),
                |t| {
                    t.query()
                        .falloff_x(24, d as u32, None, FalloffPattern::Linear, Max)
                        .falloff_y(24, d as u32, None, FalloffPattern::Linear, Max)
                        .falloff_f(24, d as u32, None, FalloffPattern::Linear, Max)
                        .raw_run()
                        .unwrap()
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

criterion_group!(benches, bench_falloff);
criterion_main!(benches);
