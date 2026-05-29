use criterion::{Criterion, criterion_group, criterion_main};
use kasane_logic::{Coordinate, Ecef};
use rand::Rng;
use std::hint::black_box;

fn bench_point_conversions(c: &mut Criterion) {
    let mut rng = rand::rng();
    let num_points = 1000;

    let mut coords = Vec::with_capacity(num_points);
    for _ in 0..num_points {
        let lat = rng.random_range(-85.0..85.0);
        let lon = rng.random_range(-180.0..180.0);
        let alt = rng.random_range(0.0..1000.0);
        coords.push(Coordinate::new(lat, lon, alt).unwrap());
    }

    let ecefs: Vec<Ecef> = coords.iter().map(|&c| c.into()).collect();

    let mut group = c.benchmark_group("Point_Conversion");

    group.bench_function("Coordinate_to_Ecef", |b| {
        b.iter(|| {
            for &coord in &coords {
                let ecef: Ecef = coord.into();
                black_box(ecef);
            }
        });
    });

    group.bench_function("Ecef_to_Coordinate", |b| {
        b.iter(|| {
            for &ecef in &ecefs {
                let coord: Coordinate = ecef.try_into().unwrap();
                black_box(coord);
            }
        });
    });

    group.bench_function("Coordinate_to_SingleId_Z20", |b| {
        b.iter(|| {
            for &coord in &coords {
                let id = coord.single_id(20).unwrap();
                black_box(id);
            }
        });
    });

    group.bench_function("Ecef_to_SingleId_Z20", |b| {
        b.iter(|| {
            for &ecef in &ecefs {
                let id = ecef.single_id(20).unwrap();
                black_box(id);
            }
        });
    });

    group.finish();
}

criterion_group!(benches, bench_point_conversions);
criterion_main!(benches);
