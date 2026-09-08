use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};

#[path = "../cases.rs"]
mod cases;
#[path = "../utils.rs"]
mod utils;

fn bench_falloff(c: &mut Criterion) {
    let mut group = c.benchmark_group("Unary/Falloff");
    group.sample_size(10);
    group.warm_up_time(std::time::Duration::from_millis(500));
    group.measurement_time(std::time::Duration::from_secs(1));

    let table = utils::get_full_data();

    for &dim in &["x", "y", "f", "all"] {
        for &dist in &cases::DEFAULT_DISTANCES {
            group.bench_with_input(
                BenchmarkId::new(format!("falloff_{}", dim), dist),
                &dist,
                |b, &d| {
                    b.iter_batched(
                        || table.clone(),
                        |t| cases::falloff_by_dim(t, dim, d as u32).run().unwrap().count(),
                        BatchSize::SmallInput,
                    );
                },
            );
        }
    }

    group.finish();
}

criterion_group!(benches, bench_falloff);
criterion_main!(benches);
