use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use kasane_logic::{Coordinate, Geometry, Triangle};
use std::hint::black_box;

/// ズームレベルによる負荷の変化を計測
fn big_triangle_z_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("ZoomLeveL Scaling (Large Triangle)");

    //入力されるズームレベルの一覧
    let inputs = 15..=25;

    //十分に大きな三角形
    let tokyo = Coordinate::new(35.681382, 139.766083, 0.0).unwrap();
    let ikebukuro = Coordinate::new(35.728926, 139.71038, 100.0).unwrap();
    let shinagawa = Coordinate::new(35.630152, 139.74044, 800.0).unwrap();
    let triangle = Triangle::new([tokyo, ikebukuro, shinagawa]);

    //繰り返しで処理
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
