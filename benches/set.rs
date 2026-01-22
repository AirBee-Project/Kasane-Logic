use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use kasane_logic::{SetOnMemory, SingleId};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::hint::black_box;

fn generate_fixed_ids(size: usize, seed: u64) -> Vec<SingleId> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed); // シード固定
    let mut ids = Vec::with_capacity(size);

    for _ in 0..size {
        let id = SingleId::random_within_using(&mut rng, 0..=10);
        ids.push(id);
    }
    ids
}

fn build_set_from_ids(ids: &[SingleId]) -> SetOnMemory {
    let mut set = SetOnMemory::default();
    for id in ids {
        set.insert(id);
    }
    set
}

fn bench_set_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Set Operations");

    let sizes = [100, 1_000, 10_000];

    for &size in &sizes {
        let ids_a = generate_fixed_ids(size, 12345);
        let ids_b = generate_fixed_ids(size, 67890);

        let set_a = build_set_from_ids(&ids_a);
        let set_b = build_set_from_ids(&ids_b);

        group.bench_with_input(BenchmarkId::new("Insert", size), &ids_a, |b, ids| {
            b.iter_batched(
                || SetOnMemory::default(),
                |mut set| {
                    // Routine
                    for id in ids {
                        set.insert(id);
                    }
                    black_box(set)
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_with_input(
            BenchmarkId::new("Union", size),
            &(&set_a, &set_b),
            |b, (a, b_set)| {
                b.iter(|| {
                    let result = a.union(b_set);
                    black_box(result)
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Intersection", size),
            &(&set_a, &set_b),
            |b, (a, b_set)| {
                b.iter(|| {
                    let result = a.intersection(b_set);
                    black_box(result)
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Difference", size),
            &(&set_a, &set_b),
            |b, (a, b_set)| {
                b.iter(|| {
                    let result = a.difference(b_set);
                    black_box(result)
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_set_operations);
criterion_main!(benches);
