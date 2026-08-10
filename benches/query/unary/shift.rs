use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use kasane_logic::Source;

#[path = "../utils.rs"]
mod utils;

fn bench_shift(c: &mut Criterion) {
    let mut group = c.benchmark_group("Unary/Shift");
    group.sample_size(10);
    group.warm_up_time(std::time::Duration::from_millis(500));
    group.measurement_time(std::time::Duration::from_secs(1));

    let table = utils::get_full_data();
    let distances = [1, 3, 5, 7, 10, 12, 15];

    // 個別の次元で行う関数 (X)
    for &dist in &distances {
        group.bench_with_input(BenchmarkId::new("shift_x", dist), &dist, |b, &d| {
            b.iter_batched(
                || table.clone(),
                |t| t.query().shift_x(24, d).raw_run().unwrap(),
                BatchSize::SmallInput,
            );
        });
    }

    // 個別の次元で行う関数 (Y)
    for &dist in &distances {
        group.bench_with_input(BenchmarkId::new("shift_y", dist), &dist, |b, &d| {
            b.iter_batched(
                || table.clone(),
                |t| t.query().shift_y(24, d).raw_run().unwrap(),
                BatchSize::SmallInput,
            );
        });
    }

    // 個別の次元で行う関数 (F)
    for &dist in &distances {
        group.bench_with_input(BenchmarkId::new("shift_f", dist), &dist, |b, &d| {
            b.iter_batched(
                || table.clone(),
                |t| t.query().shift_f(24, d).raw_run().unwrap(),
                BatchSize::SmallInput,
            );
        });
    }

    // 全ての次元で行う関数
    for &dist in &distances {
        group.bench_with_input(BenchmarkId::new("shift_all", dist), &dist, |b, &d| {
            b.iter_batched(
                || table.clone(),
                |t| {
                    t.query()
                        .shift_x(24, d)
                        .shift_y(24, -d)
                        .shift_f(24, d)
                        .raw_run()
                        .unwrap()
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

criterion_group!(benches, bench_shift);
criterion_main!(benches);
