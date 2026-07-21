//! 実際のビルデータを用いたワークフローベンチマーク。
//!
//! sample/bldg_risk.json（約11MB）から一部のデータを抽出し、
//! ・移動（Shift）
//! ・値伝播（Falloff）
//! ・移動と値伝播（Shift + Falloff）
//! を現実的な実行時間で計測する。

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use kasane_logic::{
    SpatialIdCollection, SpatialIdTable, ZoomLevel,
    merge_policy::{Average, Max},
};
use std::fs;
use std::sync::OnceLock;

static FULL_DATA: OnceLock<SpatialIdTable<u32>> = OnceLock::new();

/// ベンチマーク起動時に一度だけ JSON を読み込んでパースする
fn get_full_data() -> &'static SpatialIdTable<u32> {
    FULL_DATA.get_or_init(|| {
        let json_str = fs::read_to_string("sample/bldg_risk.json")
            .expect("Failed to read sample/bldg_risk.json. Make sure you run from workspace root.");
        serde_json::from_str(&json_str).expect("Failed to parse JSON")
    })
}

/// N件のサブセットを生成する。データが N件未満なら全件を返す。
fn get_subset(n: usize) -> SpatialIdTable<u32> {
    let full = get_full_data();
    let mut subset = SpatialIdTable::new();
    for (id, &val) in full.iter().take(n) {
        subset.insert(id.clone(), val);
    }
    subset
}

/// スケーリングベンチの共通枠。
fn bench_workflow<F>(c: &mut Criterion, group_name: &str, sizes: &[usize], op: F)
where
    F: Fn(SpatialIdTable<u32>) -> SpatialIdTable<u32>,
{
    let mut group = c.benchmark_group(group_name);
    // 重い処理なので sample_size を小さめにして時間を節約
    group.sample_size(10);

    for &n in sizes {
        let table = get_subset(n);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &table, |b, table| {
            b.iter_batched(|| table.clone(), &op, BatchSize::SmallInput);
        });
    }
    group.finish();
}

fn bench_shift(c: &mut Criterion) {
    bench_workflow(c, "Workflow/BldgRisk_Shift", &[100, 1000, 5000], |t| {
        t.query().shift_x(24, 5).shift_y(24, -5).raw_run().unwrap()
    });
}

fn bench_falloff(c: &mut Criterion) {
    // Falloffは乗数的に重くなるため、少し小さめのNで検証
    bench_workflow(c, "Workflow/BldgRisk_Falloff", &[10, 100, 1000], |t| {
        t.query()
            .falloff_linear_x(24, 5, Max)
            .falloff_linear_y(24, 5, Max)
            .falloff_linear_f(24, 15, Max)
            .raw_run()
            .unwrap()
    });
}

fn bench_shift_and_falloff(c: &mut Criterion) {
    bench_workflow(c, "Workflow/BldgRisk_ShiftFalloff", &[10, 100, 1000], |t| {
        t.query()
            .shift_x(24, 5)
            .shift_y(24, -5)
            .falloff_linear_x(24, 5, Max)
            .falloff_linear_y(24, 5, Max)
            .falloff_linear_f(24, 15, Max)
            .raw_run()
            .unwrap()
    });
}

fn bench_zoom_out(c: &mut Criterion) {
    bench_workflow(
        c,
        "Workflow/BldgRisk_ZoomOut_Average",
        &[100, 1000, 5000],
        |t| {
            t.query()
                .zoom_out(ZoomLevel::new(18).unwrap(), Average)
                .raw_run()
                .unwrap()
        },
    );
}

criterion_group!(
    benches,
    bench_shift,
    bench_falloff,
    bench_shift_and_falloff,
    bench_zoom_out
);
criterion_main!(benches);
