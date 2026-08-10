use core::ops::Bound;
use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use kasane_logic::Source;
use kasane_logic::spatial_id::collection::query::ops::unary::filter_values::ValuePredicate;

#[path = "../utils.rs"]
mod utils;

fn bench_filter_values(c: &mut Criterion) {
    let mut group = c.benchmark_group("Unary/FilterValues");
    group.sample_size(10);

    let table = utils::get_full_data();
    group.bench_function("filter_values_in_range", |b| {
        b.iter_batched(
            || table.clone(),
            |t| {
                // リスク値(3以上)を抽出する
                let predicate = ValuePredicate::InRange(Bound::Included(3), Bound::Unbounded);
                t.query().filter_values(predicate).raw_run().unwrap()
            },
            BatchSize::SmallInput,
        );
    });
    group.finish();
}

criterion_group!(benches, bench_filter_values);
criterion_main!(benches);
