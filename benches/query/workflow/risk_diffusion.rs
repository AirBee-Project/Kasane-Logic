use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use kasane_logic::SpatialIdTable;

#[path = "../cases.rs"]
mod cases;
#[path = "../utils.rs"]
mod utils;

fn run_query(table: SpatialIdTable<u32>) -> usize {
    cases::risk_diffusion(table, 10).run().unwrap().count()
}

fn bench_risk_diffusion(c: &mut Criterion) {
    let mut group = c.benchmark_group("Workflow/RiskDiffusion");
    group.sample_size(10);
    group.warm_up_time(std::time::Duration::from_millis(500));
    group.measurement_time(std::time::Duration::from_secs(1));

    let table = utils::get_full_data();

    group.bench_function("custom_query", |b| {
        b.iter_batched(|| table.clone(), run_query, BatchSize::SmallInput);
    });
    group.finish();
}

criterion_group!(benches, bench_risk_diffusion);
criterion_main!(benches);
