//! SpatialIdSet::remove ベンチマーク。
//! セット構築はセットアップ（計測外）とし、Remove のコストのみを計測する。

#[path = "patterns.rs"]
#[allow(dead_code)]
mod patterns;

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use patterns::{COUNTS, PATTERNS, Z, build_set};
use std::hint::black_box;

fn bench_remove(c: &mut Criterion) {
    for &(name, pattern_fn) in PATTERNS {
        let mut group = c.benchmark_group(format!("SpatialIdSet/Remove/{name}"));
        for &count in COUNTS {
            let ids = pattern_fn(Z, count);
            group.bench_with_input(BenchmarkId::from_parameter(count), &ids, |b, ids| {
                b.iter_batched(
                    || build_set(ids),
                    |mut set| {
                        for id in ids {
                            let _ = set.remove(id);
                        }
                        black_box(set.is_empty())
                    },
                    BatchSize::SmallInput,
                );
            });
        }
        group.finish();
    }
}

criterion_group!(benches, bench_remove);
criterion_main!(benches);
