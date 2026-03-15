use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use kasane_logic::{Coordinate, Geometry, Triangle};
use std::hint::black_box;

/// ズームレベルによる負荷の変化を計測
fn big_triangle_z_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("ZoomLeveL Scaling (Small Triangle)");

    let inputs = 15..=25;

    // 東京駅 八重洲口付近
    let tokyo_station = Coordinate::new(35.681000, 139.767000, 0.0).unwrap();
    let point_b = Coordinate::new(35.681200, 139.767200, 10.0).unwrap();
    let point_c = Coordinate::new(35.680700, 139.767050, 5.0).unwrap();

    let triangle = Triangle::new([tokyo_station, point_b, point_c]);

    for z in inputs {
        group.bench_with_input(BenchmarkId::new("Z", z), &z, |b, &z| {
            b.iter(|| {
                let iter = triangle
                    .single_ids(z as u8)
                    .expect("Failed to get iterator");
                black_box(iter.count())
            });
        });
    }
    group.finish();
}

criterion_group!(benches, big_triangle_z_bench);
criterion_main!(benches);
