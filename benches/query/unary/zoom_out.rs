use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};

#[path = "../cases.rs"]
mod cases;
#[path = "../utils.rs"]
mod utils;

fn bench_zoom_out(c: &mut Criterion) {
    let mut group = c.benchmark_group("Unary/ZoomOut");
    group.sample_size(10);
    group.warm_up_time(std::time::Duration::from_millis(500));
    group.measurement_time(std::time::Duration::from_secs(1));

    let table = utils::get_full_data();

    for &level in &cases::ZOOM_LEVELS {
        group.bench_with_input(BenchmarkId::new("zoom_out_to", level), &level, |b, &lvl| {
            b.iter_batched(
                || table.clone(),
                |t| cases::zoom_out(t, lvl).run().unwrap().count(),
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

criterion_group!(benches, bench_zoom_out);
criterion_main!(benches);
