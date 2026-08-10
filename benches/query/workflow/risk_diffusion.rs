use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use kasane_logic::Source;
use kasane_logic::merge_policy::Max;
use kasane_logic::spatial_id::collection::query::ops::unary::falloff::FalloffPattern;

#[path = "../utils.rs"]
mod utils;

fn bench_risk_diffusion(c: &mut Criterion) {
    let mut group = c.benchmark_group("Workflow/RiskDiffusion");
    group.sample_size(10);

    let table = utils::get_full_data();

    // 生成結果をファイルに保存する（ベンチマーク計測外で一度だけ実行）
    let diffused = table
        .clone()
        .query()
        .shift_x(24, 5)
        .shift_y(24, -5)
        .falloff_x(24, 5, None, FalloffPattern::Linear, Max)
        .falloff_y(24, 5, None, FalloffPattern::Linear, Max)
        .falloff_f(24, 15, None, FalloffPattern::Linear, Max)
        .raw_run()
        .unwrap();
    let result = table
        .clone()
        .query()
        .merge(diffused.query(), 0, Max)
        .raw_run()
        .unwrap();
    utils::save_result_json("risk_diffusion", &result);
    group.bench_function("shift_falloff_merge", |b| {
        b.iter_batched(
            || table.clone(),
            |t| {
                // 移動と減衰を適用後、元のデータと結合する
                let diffused = t
                    .clone()
                    .query()
                    .shift_x(24, 5)
                    .shift_y(24, -5)
                    .falloff_x(24, 5, None, FalloffPattern::Linear, Max)
                    .falloff_y(24, 5, None, FalloffPattern::Linear, Max)
                    .falloff_f(24, 15, None, FalloffPattern::Linear, Max)
                    .raw_run()
                    .unwrap();

                t.query().merge(diffused.query(), 0, Max).raw_run().unwrap()
            },
            BatchSize::SmallInput,
        );
    });
    group.finish();
}

criterion_group!(benches, bench_risk_diffusion);
criterion_main!(benches);
