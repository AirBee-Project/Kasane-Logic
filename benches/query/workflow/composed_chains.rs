use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use kasane_logic::{
    Side::Upper, Source, SpatialIdTable, merge_policy::Max,
    spatial_id::collection::query::ops::unary::falloff::FalloffPattern,
};

#[path = "../utils.rs"]
mod utils;

fn bench_composed_chains(c: &mut Criterion) {
    let mut group = c.benchmark_group("Workflow/ComposedChains");
    group.sample_size(10);
    group.warm_up_time(std::time::Duration::from_millis(500));
    group.measurement_time(std::time::Duration::from_secs(1));

    let table = utils::get_full_data();

    let dist = 10;

    group.bench_function("shift_extrude", |b| {
        b.iter_batched(
            || table.clone(),
            |t| {
                t.query()
                    .shift_x(24, dist)
                    .extrude_y(24, 0, 5, Max)
                    .raw_run()
                    .unwrap()
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("shift_falloff", |b| {
        b.iter_batched(
            || table.clone(),
            |t| {
                t.query()
                    .shift_x(24, dist)
                    .falloff_f(25, 5, Some(Upper), FalloffPattern::Linear, Max)
                    .raw_run()
                    .unwrap()
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("shift_zoomout_extrude", |b| {
        b.iter_batched(
            || table.clone(),
            |t| {
                t.query()
                    .shift_x(24, dist)
                    .zoom_out(22, Max)
                    .extrude_y(22, 0, 2, Max)
                    .raw_run()
                    .unwrap()
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(benches, bench_composed_chains);
criterion_main!(benches);
