use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;
use kasane_logic::{Coordinate, CoverSingleIds, geometry::shape::tube::Tube};

fn bench_tube(c: &mut Criterion) {
    let base_lat = 35.681000;
    let base_lon = 139.767000;
    let base_alt = 0.0;

    // 1. ZoomLevel Scaling (Fixed Size: 2 points, approx 111m long, r=30m)
    let mut zoom_group = c.benchmark_group("Tube_ZoomLevel_Scaling");
    for z in 15..=25 {
        zoom_group.bench_with_input(BenchmarkId::from_parameter(z), &z, |b, &z| {
            b.iter(|| {
                let p1 = Coordinate::new(base_lat, base_lon, base_alt).unwrap();
                let p2 = Coordinate::new(base_lat + 0.001, base_lon, base_alt).unwrap();
                let tube = Tube::new(vec![p1, p2], 30.0).unwrap();
                let iter = tube.cover_single_ids(z as u8).expect("Failed to get iterator");
                black_box(iter.count());
            });
        });
    }
    zoom_group.finish();

    // 2. Size Scaling (Fixed Zoom: z=20)
    // Scale length of the tube
    let mut size_group = c.benchmark_group("Tube_Size_Scaling");
    let scales = [0.0001, 0.0005, 0.001, 0.005, 0.01];
    for &scale in &scales {
        size_group.bench_with_input(BenchmarkId::from_parameter(scale), &scale, |b, &scale| {
            b.iter(|| {
                let p1 = Coordinate::new(base_lat, base_lon, base_alt).unwrap();
                let p2 = Coordinate::new(base_lat + scale, base_lon + scale, base_alt).unwrap();
                let tube = Tube::new(vec![p1, p2], 30.0).unwrap();
                let iter = tube.cover_single_ids(20).expect("Failed to get iterator");
                black_box(iter.count());
            });
        });
    }
    size_group.finish();
}

criterion_group!(benches, bench_tube);
criterion_main!(benches);
