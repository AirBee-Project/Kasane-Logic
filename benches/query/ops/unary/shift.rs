//! 単項クエリ演算子（shift）のベンチマーク。

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use kasane_logic::{SingleId, Source, SpatialIdTable};

const OP_ZOOM: u8 = 25;

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
        t.query().shift_x(OP_ZOOM, 10).raw_run_into().unwrap()
    });
}

fn bench_shift_x_y_chained(c: &mut Criterion) {
    bench_scaling(c, "UnaryOps/shift_x_then_y", &[1, 10, 50, 100], |t| {
        t.query()
            .shift_x(OP_ZOOM, 10)
            .shift_y(OP_ZOOM, 10)
            .raw_run_into()
            .unwrap()
    });
}

criterion_group!(benches, bench_shift_x, bench_shift_x_y_chained);
criterion_main!(benches);
