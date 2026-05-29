//! SpatialIdSet::get ベンチマーク。
//! 挿入済みセットに対して全 ID を点クエリし、ヒット数を計測する。

#[path = "patterns.rs"]
#[allow(dead_code)]
mod patterns;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use patterns::{COUNTS, PATTERNS, Z, build_set};
use std::hint::black_box;

fn bench_get(c: &mut Criterion) {
    for &(name, pattern_fn) in PATTERNS {
        let mut group = c.benchmark_group(format!("SpatialIdSet/Get/{name}"));
        for &count in COUNTS {
            let ids = pattern_fn(Z, count);
            let set = build_set(&ids);
            group.bench_with_input(BenchmarkId::from_parameter(count), &ids, |b, ids| {
                b.iter(|| {
                    let mut total = 0usize;
                    for id in ids {
                        total += set.get(id).count();
                    }
                    black_box(total)
                });
            });
        }
        group.finish();
    }
}

criterion_group!(benches, bench_get);
criterion_main!(benches);
