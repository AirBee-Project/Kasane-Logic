//! 単項クエリ演算子（falloff_linear）のベンチマーク。
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use kasane_logic::{
    SingleId, SpatialIdCollection, SpatialIdTable, spatial_id::collection::query::merge_policy::Max,
};

const OP_ZOOM: u8 = 25;
const FALLOFF_RADIUS: u32 = 8;

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

fn bench_falloff_x(c: &mut Criterion) {
    bench_scaling(c, "UnaryOps/falloff_linear_x", &[1, 10, 50, 100], |t| {
        t.query()
            .falloff_linear_x(OP_ZOOM, FALLOFF_RADIUS, Max)
            .raw_run()
            .unwrap()
    });
}

fn bench_falloff_x_y_chained(c: &mut Criterion) {
    // 連鎖は中間セルが軸ごとに膨らむため、実行時間を抑えて最大 2500 セルまでに留める。
    bench_scaling(c, "UnaryOps/falloff_linear_x_then_y", &[1, 10, 50], |t| {
        t.query()
            .falloff_linear_x(OP_ZOOM, FALLOFF_RADIUS, Max)
            .falloff_linear_y(OP_ZOOM, FALLOFF_RADIUS, Max)
            .raw_run()
            .unwrap()
    });
}

criterion_group!(benches, bench_falloff_x, bench_falloff_x_y_chained);
criterion_main!(benches);
