use core::ops::Bound;
use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use kasane_logic::Source;
use kasane_logic::spatial_id::collection::query::ops::unary::filter_values::ValuePredicate;

#[path = "../utils.rs"]
mod utils;

fn bench_filter_values(c: &mut Criterion) {
    let mut group = c.benchmark_group("Unary/FilterValues");
    group.sample_size(10);

    let table = utils::get_full_data();
    let thresholds = [1, 2, 3, 4, 5];

    for &threshold in &thresholds {
        group.bench_with_input(
            BenchmarkId::new("greater_than_or_equal", threshold),
            &threshold,
            |b, &t_val| {
                b.iter_batched(
                    || table.clone(),
                    |t| {
                        let predicate =
                            ValuePredicate::InRange(Bound::Included(t_val), Bound::Unbounded);
                        t.query().filter_values(predicate).raw_run().unwrap()
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_filter_values);
criterion_main!(benches);
