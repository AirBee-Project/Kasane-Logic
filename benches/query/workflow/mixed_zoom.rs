//! ズームが混在する演算チェーンのベンチマーク。
//!
//! `shift_x`/`shift_y` はデータと同じ z=20 で動く（安い）が、途中に挟んだ
//! `falloff_linear_f` だけがずっと深い z=27 を要求する。グリッド経路は連続する区間を
//! 平坦化する際、含まれる演算のうち最も深いズームに全体を合わせる必要があるため、
//! 区切らずに1バッチとして扱うと、安いはずの shift まで z=27 相当の見積もりコストを
//! 払うことになり、予算超過でチェーン全体が（本来グリッドに載るはずの部分も含めて）
//! 木経路にフォールバックしてしまう。
//!
//! `run_unary_chain` はズームが上がる境目でバッチを区切るので、shift の区間は
//! 安い z=20 のまま平坦化し、falloff だけが z=27 のコストを払う。

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use kasane_logic::{SingleId, Source, SpatialIdTable, merge_policy::Max};

const BASE_ZOOM: u8 = 20;
const DEEP_ZOOM: u8 = 27;

/// 値をセルごとに変えて木の圧縮（同値の隣接ブロックが1つの粗い葉へ畳まれる）を防ぐ。
/// 一律の値だと大部分が畳まれてしまい、平坦化のコストが実際の見積もりより
/// 小さくなってズームを混ぜた効果が見えにくくなる。
fn setup_cluster(n: u32) -> SpatialIdTable<u32> {
    let mut table = SpatialIdTable::new();
    for i in 0..n {
        for j in 0..n {
            let id = SingleId::new(BASE_ZOOM, 0, 931386 + i, 412903 + j).unwrap();
            table.insert(id, i * n + j);
        }
    }
    table
}

fn bench_scaling<F>(c: &mut Criterion, group_name: &str, sizes: &[u32], op: F)
where
    F: Fn(SpatialIdTable<u32>) -> SpatialIdTable<u32>,
{
    let mut group = c.benchmark_group(group_name);
    for &n in sizes {
        let voxels = (n * n) as u64;
        let table = setup_cluster(n);
        group.throughput(Throughput::Elements(voxels));
        group.bench_with_input(BenchmarkId::from_parameter(voxels), &table, |b, table| {
            b.iter_batched(|| table.clone(), &op, BatchSize::SmallInput);
        });
    }
    group.finish();
}

/// z=20のshift 2つ→z=27のfalloff 1つ→z=20のshift 2つ、という混在チェーン。
fn bench_mixed_zoom_chain(c: &mut Criterion) {
    bench_scaling(
        c,
        "Workflow/MixedZoom_ShiftFalloffShift",
        &[1, 10, 50, 100],
        |t| {
            t.query()
                .shift_x(BASE_ZOOM, 3)
                .shift_y(BASE_ZOOM, -3)
                .falloff_linear_f(DEEP_ZOOM, 2, Max)
                .shift_x(BASE_ZOOM, 1)
                .shift_y(BASE_ZOOM, -1)
                .raw_run_table()
                .unwrap()
        },
    );
}

/// 比較対象: 同じ演算だが全部z=20（ズームが混在しない）。
/// バッチが区切られてもコスト自体は変わらないはずの基準線。
fn bench_uniform_zoom_chain(c: &mut Criterion) {
    bench_scaling(
        c,
        "Workflow/MixedZoom_UniformZoomBaseline",
        &[1, 10, 50, 100],
        |t| {
            t.query()
                .shift_x(BASE_ZOOM, 3)
                .shift_y(BASE_ZOOM, -3)
                .falloff_linear_f(BASE_ZOOM, 2, Max)
                .shift_x(BASE_ZOOM, 1)
                .shift_y(BASE_ZOOM, -1)
                .raw_run_table()
                .unwrap()
        },
    );
}

criterion_group!(benches, bench_mixed_zoom_chain, bench_uniform_zoom_chain);
criterion_main!(benches);
