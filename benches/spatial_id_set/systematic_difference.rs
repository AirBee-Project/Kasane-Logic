//! SpatialIdSet の Difference（-）ベンチマーク。
//! 50% 要素重複を持つ 2 セットで計測する。

#[path = "patterns.rs"]
#[allow(dead_code)]
mod patterns;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use patterns::{COUNTS, PATTERNS, Z, make_pair};
use std::hint::black_box;

fn bench_difference(c: &mut Criterion) {
    for &(name, pattern_fn) in PATTERNS {
        let mut group = c.benchmark_group(format!("SpatialIdSet/Difference/{name}"));
        for &count in COUNTS {
            let (a, b) = make_pair(pattern_fn, Z, count);
            group.bench_with_input(BenchmarkId::from_parameter(count), &count, |bench, _| {
                bench.iter(|| black_box(&a - &b));
            });
        }
        group.finish();
    }
}

criterion_group!(benches, bench_difference);
criterion_main!(benches);
