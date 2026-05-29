use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use kasane_logic::{Coordinate, CoverSingleIds, geometry::shape::cylinder::Cylinder};
use std::hint::black_box;

fn bench_cylinder(c: &mut Criterion) {
    let base_lat = 35.681000;
    let base_lon = 139.767000;
    let base_alt = 0.0;

    // 1. ZoomLevel Scaling (Fixed Size: r=30m, height=100m)
    let mut zoom_group = c.benchmark_group("Cylinder_ZoomLevel_Scaling");
    for z in 15..=25 {
        zoom_group.bench_with_input(BenchmarkId::from_parameter(z), &z, |b, &z| {
            b.iter(|| {
                let center = Coordinate::new(base_lat, base_lon, base_alt).unwrap();
                let end = Coordinate::new(base_lat, base_lon, base_alt + 100.0).unwrap();
                let cylinder = Cylinder::new(center, end, 30.0).unwrap();
                let iter = cylinder
                    .cover_single_ids(z as u8)
                    .expect("Failed to get iterator");
                black_box(iter.count());
            });
        });
    }
    zoom_group.finish();

    // 2. Size Scaling (Fixed Zoom: z=20)
    let mut size_group = c.benchmark_group("Cylinder_Size_Scaling");
    let radiuses = [10.0, 30.0, 50.0, 100.0];
    for &r in &radiuses {
        size_group.bench_with_input(BenchmarkId::from_parameter(r), &r, |b, &r| {
            b.iter(|| {
                let center = Coordinate::new(base_lat, base_lon, base_alt).unwrap();
                let end = Coordinate::new(base_lat, base_lon, base_alt + r * 2.0).unwrap();
                let cylinder = Cylinder::new(center, end, r).unwrap();
                let iter = cylinder
                    .cover_single_ids(20)
                    .expect("Failed to get iterator");
                black_box(iter.count());
            });
        });
    }
    size_group.finish();
}

criterion_group!(benches, bench_cylinder);
criterion_main!(benches);
