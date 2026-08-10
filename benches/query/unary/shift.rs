use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use kasane_logic::Source;

#[path = "../utils.rs"]
mod utils;

fn bench_shift(c: &mut Criterion) {
    let mut group = c.benchmark_group("Unary/Shift");
    group.sample_size(10);

    let table = utils::get_full_data();

    // 個別の次元で行う関数 (X, Y, F)
    group.bench_function("shift_x", |b| {
        b.iter_batched(
            || table.clone(),
            |t| t.query().shift_x(24, 5).raw_run().unwrap(),
            BatchSize::SmallInput,
        );
    });

    group.bench_function("shift_y", |b| {
        b.iter_batched(
            || table.clone(),
            |t| t.query().shift_y(24, 5).raw_run().unwrap(),
            BatchSize::SmallInput,
        );
    });

    group.bench_function("shift_f", |b| {
        b.iter_batched(
            || table.clone(),
            |t| t.query().shift_f(24, 5).raw_run().unwrap(),
            BatchSize::SmallInput,
        );
    });

    // 全ての次元で行う関数
    group.bench_function("shift_all", |b| {
        b.iter_batched(
            || table.clone(),
            |t| {
                t.query()
                    .shift_x(24, 5)
                    .shift_y(24, -5)
                    .shift_f(24, 2)
                    .raw_run()
                    .unwrap()
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(benches, bench_shift);
criterion_main!(benches);
