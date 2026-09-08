use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};

#[path = "../cases.rs"]
mod cases;
#[path = "../utils.rs"]
mod utils;

fn bench_extrude(c: &mut Criterion) {
    let mut group = c.benchmark_group("Unary/Extrude");
    group.sample_size(10);
    group.warm_up_time(std::time::Duration::from_millis(500));
    group.measurement_time(std::time::Duration::from_secs(1));

    let table = utils::get_full_data();

    for &dim in &["x", "y", "f", "all"] {
        for &dist in &cases::DEFAULT_DISTANCES {
            group.bench_with_input(
                BenchmarkId::new(format!("extrude_{}", dim), dist),
                &dist,
                |b, &d| {
                    b.iter_batched(
                        || table.clone(),
                        |t| cases::extrude_by_dim(t, dim, d).run().unwrap().count(),
                        BatchSize::SmallInput,
                    );
                },
            );
        }
    }

    group.finish();
}

criterion_group!(benches, bench_extrude);
criterion_main!(benches);
