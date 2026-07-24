//! 単項クエリ演算子（zoom_out）のベンチマーク。

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use kasane_logic::{
    SingleId, Source, SpatialIdTable, ZoomLevel,
    merge_policy::{Average, Max},
};

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

fn bench_zoom_out_average(c: &mut Criterion) {
    bench_scaling(c, "UnaryOps/zoom_out_average_z19", &[1, 10, 50, 100], |t| {
        t.query()
            .zoom_out(ZoomLevel::new(19).unwrap(), Average)
            .raw_run()
            .unwrap()
    });
}

fn bench_zoom_out_max(c: &mut Criterion) {
    bench_scaling(c, "UnaryOps/zoom_out_max_z15", &[1, 10, 50, 100], |t| {
        t.query()
            .zoom_out(ZoomLevel::new(15).unwrap(), Max)
            .raw_run()
            .unwrap()
    });
}

criterion_group!(benches, bench_zoom_out_average, bench_zoom_out_max);
criterion_main!(benches);
