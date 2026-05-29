use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use kasane_logic::{Coordinate, CoverSingleIds, Line};
use std::hint::black_box;

fn bench_line(c: &mut Criterion) {
    let base_lat = 35.681000;
    let base_lon = 139.767000;
    let base_alt = 0.0;

    // 1. ZoomLevel Scaling (Fixed Size: approx 111m)
    let mut zoom_group = c.benchmark_group("Line_ZoomLevel_Scaling");
    for z in 15..=25 {
        zoom_group.bench_with_input(BenchmarkId::from_parameter(z), &z, |b, &z| {
            b.iter(|| {
                let p1 = Coordinate::new(base_lat, base_lon, base_alt).unwrap();
                let p2 = Coordinate::new(base_lat + 0.001, base_lon, base_alt).unwrap();
                let line = Line::new([p1, p2]);
                let iter = line
                    .cover_single_ids(z as u8)
                    .expect("Failed to get iterator");
                black_box(iter.count());
            });
        });
    }
    zoom_group.finish();

    // 2. Size Scaling (Fixed Zoom: z=20)
    let mut size_group = c.benchmark_group("Line_Size_Scaling");
    // scale represents approx 10m, 50m, 100m, 500m, 1000m
    let scales = [0.0001, 0.0005, 0.001, 0.005, 0.01];
    for &scale in &scales {
        size_group.bench_with_input(BenchmarkId::from_parameter(scale), &scale, |b, &scale| {
            b.iter(|| {
                let p1 = Coordinate::new(base_lat, base_lon, base_alt).unwrap();
                let p2 = Coordinate::new(base_lat + scale, base_lon + scale, base_alt).unwrap();
                let line = Line::new([p1, p2]);
                let iter = line.cover_single_ids(20).expect("Failed to get iterator");
                black_box(iter.count());
            });
        });
    }
    size_group.finish();
}

criterion_group!(benches, bench_line);
criterion_main!(benches);
