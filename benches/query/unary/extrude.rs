use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use kasane_logic::Source;
use kasane_logic::merge_policy::Max;

#[path = "../utils.rs"]
mod utils;

fn bench_extrude(c: &mut Criterion) {
    let mut group = c.benchmark_group("Unary/Extrude");
    group.sample_size(10);

    let table = utils::get_full_data();

    // 個別の次元で行う関数 (X, Y, F)
    group.bench_function("extrude_x", |b| {
        b.iter_batched(
            || table.clone(),
            |t| t.query().extrude_x(24, 0, 5, Max).raw_run().unwrap(),
            BatchSize::SmallInput,
        );
    });

    group.bench_function("extrude_y", |b| {
        b.iter_batched(
            || table.clone(),
            |t| t.query().extrude_y(24, 0, 5, Max).raw_run().unwrap(),
            BatchSize::SmallInput,
        );
    });

    group.bench_function("extrude_f", |b| {
        b.iter_batched(
            || table.clone(),
            |t| t.query().extrude_f(24, 0, 5, Max).raw_run().unwrap(),
            BatchSize::SmallInput,
        );
    });

    // 全ての次元で行う関数
    group.bench_function("extrude_all", |b| {
        b.iter_batched(
            || table.clone(),
            |t| {
                t.query()
                    .extrude_x(24, 0, 2, Max)
                    .extrude_y(24, 0, 2, Max)
                    .extrude_f(24, 0, 2, Max)
                    .raw_run()
                    .unwrap()
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(benches, bench_extrude);
criterion_main!(benches);
