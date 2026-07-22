//! 実際のビルデータを用いたLazyViewのワークフローベンチマーク。
//!
//! sample/bldg_risk.jsonから一部のデータを抽出し、重いクエリ演算
//! （例: FalloffやShift）をLazyViewで遅延評価し、特定の空間IDに対する
//! 取得が高速に行えるかを検証する。

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use kasane_logic::{
    Source, SpatialIdTable, ZoomLevel,
    merge_policy::{Average, Max},
};
use std::fs;
use std::sync::OnceLock;

static FULL_DATA: OnceLock<SpatialIdTable<u32>> = OnceLock::new();

fn get_full_data() -> &'static SpatialIdTable<u32> {
    FULL_DATA.get_or_init(|| {
        let json_str = fs::read_to_string("sample/bldg_risk.json")
            .expect("Failed to read sample/bldg_risk.json. Make sure you run from workspace root.");
        serde_json::from_str(&json_str).expect("Failed to parse JSON")
    })
}

fn get_subset(n: usize) -> SpatialIdTable<u32> {
    let full = get_full_data();
    let mut subset = SpatialIdTable::new();
    for (id, &val) in full.iter().take(n) {
        subset.insert(id.clone(), val);
    }
    subset
}

fn bench_lazy_workflow(c: &mut Criterion) {
    let mut group = c.benchmark_group("LazyView/Workflow/BldgRisk");
    group.sample_size(10); // 重い処理のためサンプルサイズを小さくする

    let sizes = [10, 100, 500]; // eager側が重くなることを想定してサイズは控えめ

    for &n in sizes.iter() {
        let table = get_subset(n);
        // 対象のIDを先頭から適当に選ぶ
        let target_id = table.iter().next().unwrap().0.clone();

        group.throughput(Throughput::Elements(1));

        // LazyView bench
        group.bench_with_input(BenchmarkId::new("lazy", n), &table, |b, t| {
            let query = t
                .clone()
                .query()
                .shift_x(24, 5)
                .shift_y(24, -5)
                .falloff_linear_x(24, 5, Max)
                .falloff_linear_y(24, 5, Max);

            let lazy = query.lazy();
            b.iter(|| lazy.get(target_id.clone()).unwrap().count());
        });

        // Eager bench (as baseline)
        group.bench_with_input(BenchmarkId::new("eager", n), &table, |b, t| {
            b.iter_batched(
                || t.clone(),
                |table_clone| {
                    let res: SpatialIdTable<u32> = table_clone
                        .query()
                        .shift_x(24, 5)
                        .shift_y(24, -5)
                        .falloff_linear_x(24, 5, Max)
                        .falloff_linear_y(24, 5, Max)
                        .raw_run_into()
                        .unwrap();
                    let _ = res.get(&target_id).count();
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_lazy_workflow_zoom_out(c: &mut Criterion) {
    let mut group = c.benchmark_group("LazyView/Workflow/BldgRisk_ZoomOut");
    group.sample_size(10);

    let sizes = [10, 100, 500];

    for &n in sizes.iter() {
        let table = get_subset(n);

        // 事前にZoomOutした結果から妥当なターゲットIDを取得
        let zoomed: SpatialIdTable<u32> = table
            .clone()
            .query()
            .zoom_out(ZoomLevel::new(18).unwrap(), Average)
            .raw_run_into()
            .unwrap();
        let target_id = zoomed.iter().next().unwrap().0.clone();

        group.throughput(Throughput::Elements(1));

        // LazyView bench
        group.bench_with_input(BenchmarkId::new("lazy", n), &table, |b, t| {
            let query = t
                .clone()
                .query()
                .zoom_out(ZoomLevel::new(18).unwrap(), Average);

            let lazy = query.lazy();
            b.iter(|| lazy.get(target_id.clone()).unwrap().count());
        });

        // Eager bench (as baseline)
        group.bench_with_input(BenchmarkId::new("eager", n), &table, |b, t| {
            b.iter_batched(
                || t.clone(),
                |table_clone| {
                    let res: SpatialIdTable<u32> = table_clone
                        .query()
                        .zoom_out(ZoomLevel::new(18).unwrap(), Average)
                        .raw_run_into()
                        .unwrap();
                    let _ = res.get(&target_id).count();
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

criterion_group!(benches, bench_lazy_workflow, bench_lazy_workflow_zoom_out);
criterion_main!(benches);
