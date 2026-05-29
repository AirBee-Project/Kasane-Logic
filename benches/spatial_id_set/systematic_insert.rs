//! SpatialIdSet::insert ベンチマーク。
//! 6 パターン × 3 サイズ、および z=15〜22 のズームスケーリングを計測する。

#[path = "patterns.rs"]
#[allow(dead_code)]
mod patterns;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use patterns::{COUNTS, PATTERNS, Z, build_set, dense_cluster};
use std::hint::black_box;

fn bench_insert(c: &mut Criterion) {
    for &(name, pattern_fn) in PATTERNS {
        let mut group = c.benchmark_group(format!("SpatialIdSet/Insert/{name}"));
        for &count in COUNTS {
            let ids = pattern_fn(Z, count);
            group.bench_with_input(BenchmarkId::from_parameter(count), &ids, |b, ids| {
                b.iter(|| black_box(build_set(ids)));
            });
        }
        group.finish();
    }
}

/// z=15〜22 を固定サイズ Dense パターンで走査し、ズームレベルの影響を単離する。
fn bench_zoom_scaling(c: &mut Criterion) {
    const FIXED_COUNT: usize = 1_000;
    let mut group = c.benchmark_group("SpatialIdSet/ZoomScaling/Dense");
    for z in [15u8, 18, 20, 22] {
        let ids = dense_cluster(z, FIXED_COUNT);
        group.bench_with_input(BenchmarkId::from_parameter(z), &ids, |b, ids| {
            b.iter(|| black_box(build_set(ids)));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_insert, bench_zoom_scaling);
criterion_main!(benches);
