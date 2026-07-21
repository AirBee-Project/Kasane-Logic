//! 遅延ビュー(LazyView)のベンチマーク。

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use kasane_logic::{SingleId, SpatialIdCollection, SpatialIdTable};

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

fn bench_lazy_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("LazyView/get");
    let sizes = [10, 20, 50]; // limit sizes to avoid being too slow in eager setup
    for &n in sizes.iter() {
        let voxels = (n * n) as u64;
        let table = setup_cluster(n);
        let target_id = SingleId::new(20, 0, 931386 + n / 2, 412903 + n / 2).unwrap();

        group.throughput(Throughput::Elements(1));

        // LazyView bench
        group.bench_with_input(BenchmarkId::new("lazy", voxels), &table, |b, t| {
            let query = t.clone().query().shift_x(OP_ZOOM, 10).shift_y(OP_ZOOM, 10);
            let lazy = query.lazy();
            b.iter(|| lazy.get(target_id.clone()).unwrap());
        });

        // Eager bench (as baseline)
        group.bench_with_input(BenchmarkId::new("eager", voxels), &table, |b, t| {
            b.iter_batched(
                || t.clone(),
                |table| {
                    let res = table
                        .query()
                        .shift_x(OP_ZOOM, 10)
                        .shift_y(OP_ZOOM, 10)
                        .raw_run()
                        .unwrap();
                    let target_flex: kasane_logic::FlexId = target_id.clone().into();
                    let _ = res.try_get(&target_flex).unwrap().count();
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

criterion_group!(benches, bench_lazy_get);
criterion_main!(benches);
