use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use kasane_logic::Source;
use kasane_logic::merge_policy::Max;
use kasane_logic::spatial_id::collection::query::ops::unary::falloff::FalloffPattern;

#[path = "../utils.rs"]
mod utils;

fn bench_falloff(c: &mut Criterion) {
    let mut group = c.benchmark_group("Unary/Falloff");
    group.sample_size(10);

    let table = utils::get_full_data();

    // 個別の次元で行う関数 (X, Y, F)
    group.bench_function("falloff_x", |b| {
        b.iter_batched(
            || table.clone(),
            |t| {
                t.query()
                    .falloff_x(24, 5, None, FalloffPattern::Linear, Max)
                    .raw_run()
                    .unwrap()
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("falloff_y", |b| {
        b.iter_batched(
            || table.clone(),
            |t| {
                t.query()
                    .falloff_y(24, 5, None, FalloffPattern::Linear, Max)
                    .raw_run()
                    .unwrap()
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("falloff_f", |b| {
        b.iter_batched(
            || table.clone(),
            |t| {
                t.query()
                    .falloff_f(24, 15, None, FalloffPattern::Linear, Max)
                    .raw_run()
                    .unwrap()
            },
            BatchSize::SmallInput,
        );
    });

    // 全ての次元で行う関数
    group.bench_function("falloff_all", |b| {
        b.iter_batched(
            || table.clone(),
            |t| {
                t.query()
                    .falloff_x(24, 5, None, FalloffPattern::Linear, Max)
                    .falloff_y(24, 5, None, FalloffPattern::Linear, Max)
                    .falloff_f(24, 15, None, FalloffPattern::Linear, Max)
                    .raw_run()
                    .unwrap()
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(benches, bench_falloff);
criterion_main!(benches);
