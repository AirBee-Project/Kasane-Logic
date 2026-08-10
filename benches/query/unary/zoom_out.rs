use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use kasane_logic::merge_policy::Average;
use kasane_logic::{Source, ZoomLevel};

#[path = "../utils.rs"]
mod utils;

fn bench_zoom_out(c: &mut Criterion) {
    let mut group = c.benchmark_group("Unary/ZoomOut");
    group.sample_size(10);

    let table = utils::get_full_data();
    group.bench_function("zoom_out_18_avg", |b| {
        b.iter_batched(
            || table.clone(),
            |t| {
                t.query()
                    .zoom_out(ZoomLevel::new(18).unwrap(), Average)
                    .raw_run()
                    .unwrap()
            },
            BatchSize::SmallInput,
        );
    });
    group.finish();
}

criterion_group!(benches, bench_zoom_out);
criterion_main!(benches);
