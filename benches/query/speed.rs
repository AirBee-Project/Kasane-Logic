//! Query エンジンの速度ベンチマーク (Criterion)。
//!
//! cases.rs で一元管理されているテストケース配列を基に、
//! 各クエリ操作の実行速度を測定する。
//!
//! 実行方法:
//!   cargo bench --bench query_speed
//!   cargo bench --bench query_speed -- Shift  # 特定グループのみ

use criterion::{Criterion, criterion_group, criterion_main};
use kasane_logic::SpatialIdTable;

mod cases;
mod utils;

fn bench_group(c: &mut Criterion, test_cases: &[cases::TestCase], table: &SpatialIdTable<u32>) {
    // グループ名はケース自身が持つ `group` を使う(同一配列内は全て同じ値である前提)。
    let group_name = test_cases[0].group;
    debug_assert!(
        test_cases.iter().all(|c| c.group == group_name),
        "all cases in a single bench_group call must share the same `group`"
    );

    let mut group = c.benchmark_group(group_name);
    group.sample_size(10);
    group.warm_up_time(std::time::Duration::from_millis(500));
    group.measurement_time(std::time::Duration::from_secs(1));

    for case in test_cases {
        group.bench_function(case.name, |b| {
            b.iter(|| case.run_stream(table));
        });
    }

    group.finish();
}

fn bench_queries(c: &mut Criterion) {
    let table = utils::get_full_data();

    bench_group(c, cases::SHIFT_CASES, table);
    bench_group(c, cases::EXTRUDE_CASES, table);
    bench_group(c, cases::FALLOFF_CASES, table);
    bench_group(c, cases::ZOOM_OUT_CASES, table);
    bench_group(c, cases::FILTER_CASES, table);
    bench_group(c, cases::WORKFLOW_CASES, table);
}

criterion_group!(benches, bench_queries);
criterion_main!(benches);
