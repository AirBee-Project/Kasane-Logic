use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use kasane_logic::{
    Side::Upper, Source, SpatialIdTable, merge_policy::Max,
    spatial_id::collection::query::cancellation::CancellationToken,
    spatial_id::collection::query::ops::unary::falloff::FalloffPattern,
};

#[path = "../utils.rs"]
mod utils;

/// ユーザー定義のクエリ。ここにベンチマークしたいクエリを一度だけ記述してください。
///
/// 結果を`SpatialIdTable`などへ集約せず`count()`で消費するのは、集約先の分だけ
/// メモリ使用量が水増しされるのを避け、クエリエンジン自体のメモリ挙動を測るため。
/// `run_by_segments`はSourceが実際に持つSegmentごとに影響範囲を求めて評価するので、
/// falloffを3段連結しても、全域をまとめて展開する`run()`より小さな単位で処理できる。
fn run_query(table: SpatialIdTable<u32>) -> usize {
    table
        .query()
        .zoom_out(22, Max)
        .falloff_f(25, 10, Some(Upper), FalloffPattern::Linear, Max)
        .falloff_x(25, 10, None, FalloffPattern::Linear, Max)
        .falloff_y(25, 10, None, FalloffPattern::Linear, Max)
        .run_by_segments(CancellationToken::never())
        .inspect(|r| {
            r.as_ref().unwrap();
        })
        .count()
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
