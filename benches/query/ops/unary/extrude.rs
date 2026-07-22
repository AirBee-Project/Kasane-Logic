//! 単項クエリ演算子（extrude）のベンチマーク。
//!
//! extrude_fは列（X,Y footprint）ごとにまとめて展開する実装になっているため、
//! 「同じ(X,Y)列を多数のセルが共有するケース」（列単位resolve_manyが効く）と
//! 「列がほぼ互いに素なケース」（従来と同程度になるはず）の両方を計測する。

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use kasane_logic::{SingleId, Source, SpatialIdTable, merge_policy::Max};

const OP_ZOOM: u8 = 25;

/// n×n個の(X,Y)列それぞれに、異なるFを持つセルをstack個スタックして積む。
/// 列単位のresolve_manyが効くケース。
fn setup_stacked(n: u32, stack: i32) -> SpatialIdTable<u32> {
    let mut table = SpatialIdTable::new();
    for i in 0..n {
        for j in 0..n {
            for f in 0..stack {
                let id = SingleId::new(20, f, 931386 + i, 412903 + j).unwrap();
                table.insert(id, 100);
            }
        }
    }
    table
}

fn bench_scaling<F>(c: &mut Criterion, group_name: &str, sizes: &[u32], setup: F, stack: i32)
where
    F: Fn(u32, i32) -> SpatialIdTable<u32>,
{
    let mut group = c.benchmark_group(group_name);
    for &n in sizes {
        let voxels = (n * n * stack as u32) as u64;
        let table = setup(n, stack);
        group.throughput(Throughput::Elements(voxels));
        group.bench_with_input(BenchmarkId::from_parameter(voxels), &table, |b, table| {
            b.iter_batched(
                || table.clone(),
                |t| {
                    let r: SpatialIdTable<u32> = t
                        .query()
                        .extrude_f(OP_ZOOM, 0, 5, Max)
                        .raw_run_into()
                        .unwrap();
                    r
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_extrude_f_flat(c: &mut Criterion) {
    bench_scaling(
        c,
        "UnaryOps/extrude_f_flat_columns",
        &[10, 50, 100],
        setup_stacked,
        1,
    );
}

fn bench_extrude_f_stacked(c: &mut Criterion) {
    bench_scaling(
        c,
        "UnaryOps/extrude_f_stacked_columns",
        &[10, 50, 100],
        setup_stacked,
        20,
    );
}

criterion_group!(benches, bench_extrude_f_flat, bench_extrude_f_stacked);
criterion_main!(benches);
