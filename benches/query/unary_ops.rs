//! 単項クエリ演算子（shift / falloff_linear）のベンチマーク。
//!
//! 計測の要点:
//! - `iter_batched` で `table.clone()` を計測対象外に出す（cheap な演算で clone/構築の
//!   固定オーバーヘッドに埋もれないように）。
//! - ソース数を n×n でスケールさせる。`from_items_with_policy` の並列チャンクビルドと Table の
//!   並列 intern は `MIN_PAR_CHUNK=512` / `PARALLEL_LEAF_CUTOFF=1024` を超えて効くため、
//!   最大 1 万セル級まで振って並列パスを踏ませる。
//! - `Throughput::Elements` でソースセル数あたりのスループットを出し、O(n) 特性を見る。

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use kasane_logic::{
    SingleId, SpatialIdCollection, SpatialIdTable, spatial_id::collection::query::merge_policy::Max,
};

/// falloff の伝播半径。連鎖時の中間セル膨張（ソース × (2r+1) を軸ごとに掛ける）を現実的な
/// 実行時間に収めるための値。大きくするとマージ段より per-cell 展開プリミティブの比重が上がる。
const FALLOFF_RADIUS: u32 = 8;

/// 演算・falloff を適用するズームレベル。データ（z=20）より細かい解像度での伝播を測る。
const OP_ZOOM: u8 = 25;

/// n×n の近接ボクセル群（z=20, 値 100）を作る。n=1 は単一ボクセルのベースライン。
fn setup_cluster(n: u32) -> SpatialIdTable<u32> {
    let mut table = SpatialIdTable::new();
    for i in 0..n {
        for j in 0..n {
            let id = SingleId::new(20, 0, 931386 + i, 412903 + j).unwrap();
            table.insert(id, 100);
        }
    }
    table
}

/// スケーリングベンチの共通枠。各サイズについて、clone を計測外に出して `op` だけを計測する。
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

fn bench_shift_x(c: &mut Criterion) {
    bench_scaling(c, "UnaryOps/shift_x", &[1, 10, 50, 100], |t| {
        t.query().shift_x(OP_ZOOM, 10).run().unwrap()
    });
}

fn bench_shift_x_y_chained(c: &mut Criterion) {
    bench_scaling(c, "UnaryOps/shift_x_then_y", &[1, 10, 50, 100], |t| {
        t.query()
            .shift_x(OP_ZOOM, 10)
            .shift_y(OP_ZOOM, 10)
            .run()
            .unwrap()
    });
}

fn bench_falloff_x(c: &mut Criterion) {
    bench_scaling(c, "UnaryOps/falloff_linear_x", &[1, 10, 50, 100], |t| {
        t.query()
            .falloff_linear_x(OP_ZOOM, FALLOFF_RADIUS, Max)
            .run()
            .unwrap()
    });
}

fn bench_falloff_x_y_chained(c: &mut Criterion) {
    // 連鎖は中間セルが軸ごとに膨らむため、実行時間を抑えて最大 2500 セルまでに留める。
    bench_scaling(c, "UnaryOps/falloff_linear_x_then_y", &[1, 10, 50], |t| {
        t.query()
            .falloff_linear_x(OP_ZOOM, FALLOFF_RADIUS, Max)
            .falloff_linear_y(OP_ZOOM, FALLOFF_RADIUS, Max)
            .run()
            .unwrap()
    });
}

criterion_group!(
    benches,
    bench_shift_x,
    bench_shift_x_y_chained,
    bench_falloff_x,
    bench_falloff_x_y_chained,
);
criterion_main!(benches);
