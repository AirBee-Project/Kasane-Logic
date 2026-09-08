use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};

#[path = "../cases.rs"]
mod cases;
#[path = "../utils.rs"]
mod utils;

fn bench_filter_values(c: &mut Criterion) {
    let mut group = c.benchmark_group("Unary/FilterValues");
    group.sample_size(10);
    group.warm_up_time(std::time::Duration::from_millis(500));
    group.measurement_time(std::time::Duration::from_secs(1));

    let table = utils::get_full_data();

    for &threshold in &cases::FILTER_THRESHOLDS {
        group.bench_with_input(
            BenchmarkId::new("greater_than_or_equal", threshold),
            &threshold,
            |b, &t_val| {
                b.iter_batched(
                    || table.clone(),
                    |t| cases::filter_values(t, t_val).run().unwrap().count(),
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_filter_values);
criterion_main!(benches);
