//! Query エンジンの速度ベンチマーク (Criterion)。
//!
//! cases.rs で一元管理されているテストケース配列を基に、
//! 各クエリ操作の実行速度を測定する。
//!
//! 実行方法:
//!   cargo bench --bench query_speed
//!   cargo bench --bench query_speed -- Shift  # 特定グループのみ

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use kasane_logic::SpatialIdTable;

mod cases;
mod utils;

fn bench_group(
    c: &mut Criterion,
    group_name: &str,
    test_cases: &[cases::TestCase],
    table: &SpatialIdTable<u32>,
) {
    let mut group = c.benchmark_group(group_name);
    group.sample_size(10);
    group.warm_up_time(std::time::Duration::from_millis(500));
    group.measurement_time(std::time::Duration::from_secs(1));

    for case in test_cases {
        group.bench_function(case.name, |b| {
            b.iter_batched(
                || table.clone(),
                |t| case.run_stream(&t),
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_queries(c: &mut Criterion) {
    let table = utils::get_full_data();

    bench_group(c, "Unary/Shift", cases::SHIFT_CASES, table);
    bench_group(c, "Unary/Extrude", cases::EXTRUDE_CASES, table);
    bench_group(c, "Unary/Falloff", cases::FALLOFF_CASES, table);
    bench_group(c, "Unary/ZoomOut", cases::ZOOM_OUT_CASES, table);
    bench_group(c, "Unary/FilterValues", cases::FILTER_CASES, table);
    bench_group(c, "Workflow/RiskDiffusion", cases::WORKFLOW_CASES, table);
}

criterion_group!(benches, bench_queries);
criterion_main!(benches);
