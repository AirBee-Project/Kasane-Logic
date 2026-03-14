use codspeed_criterion_compat::{BenchmarkId, Criterion, criterion_group, criterion_main};
use kasane_logic::{Coordinate, Geometry, Triangle};
use std::hint::black_box;

/// ズームレベルによる負荷の変化を計測
fn big_triangle_z_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("ズームレンズの変化");

    // CodSpeedでは1回の計測精度が高いため sample_size は無視されますが、
    // 互換性のために残しておいて問題ありません。
    group.sample_size(40);

    let inputs = 15..=25;

    let tokyo = Coordinate::new(35.681382, 139.766083, 0.0).unwrap();
    let ikebukuro = Coordinate::new(35.728926, 139.71038, 100.0).unwrap();
    let shinagawa = Coordinate::new(35.630152, 139.74044, 800.0).unwrap();
    let triangle = Triangle::new([tokyo, ikebukuro, shinagawa]);

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
