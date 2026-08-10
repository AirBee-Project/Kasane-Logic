use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use kasane_logic::merge_policy::Average;
use kasane_logic::{Source, ZoomLevel};

#[path = "../utils.rs"]
mod utils;

fn bench_zoom_out(c: &mut Criterion) {
    let mut group = c.benchmark_group("Unary/ZoomOut");
    group.sample_size(10);

    let table = utils::get_full_data();
    // ズームレベルを24から18まで変化させる（元のデータが24と仮定）
    let levels = [24, 23, 22, 21, 20, 19, 18];

    for &level in &levels {
        group.bench_with_input(BenchmarkId::new("zoom_out_to", level), &level, |b, &lvl| {
            b.iter_batched(
                || table.clone(),
                |t| {
                    let target_level = ZoomLevel::new(lvl).unwrap();
                    t.query().zoom_out(target_level, Average).raw_run().unwrap()
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

criterion_group!(benches, bench_zoom_out);
criterion_main!(benches);
