use core::ops::Bound;
use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use kasane_logic::merge_policy::Average;
use kasane_logic::spatial_id::collection::query::ops::unary::filter_values::ValuePredicate;
use kasane_logic::{Source, ZoomLevel};

#[path = "../utils.rs"]
mod utils;

fn bench_aggregation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Workflow/Aggregation");
    group.sample_size(10);

    let table = utils::get_full_data();

    // 生成結果をファイルに保存する（ベンチマーク計測外で一度だけ実行）
    let predicate = ValuePredicate::InRange(Bound::Included(2), Bound::Unbounded);
    let result = table
        .clone()
        .query()
        .filter_values(predicate.clone())
        .zoom_out(ZoomLevel::new(18).unwrap(), Average)
        .raw_run()
        .unwrap();
    utils::save_result_json("aggregation", &result);
    group.bench_function("filter_values_zoom_out", |b| {
        b.iter_batched(
            || table.clone(),
            |t| {
                // 低リスク値を除外後、粗い解像度へ集約する
                let predicate = ValuePredicate::InRange(Bound::Included(2), Bound::Unbounded);
                t.query()
                    .filter_values(predicate)
                    .zoom_out(ZoomLevel::new(18).unwrap(), Average)
                    .raw_run()
                    .unwrap()
            },
            BatchSize::SmallInput,
        );
    });
    group.finish();
}

criterion_group!(benches, bench_aggregation);
criterion_main!(benches);
