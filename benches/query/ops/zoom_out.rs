use criterion::{Criterion, criterion_group, criterion_main};
use kasane_logic::{Source, SpatialIdTable, merge_policy::Average};
use std::fs;
use std::sync::OnceLock;

static FULL_DATA: OnceLock<SpatialIdTable<u32>> = OnceLock::new();

fn get_full_data() -> &'static SpatialIdTable<u32> {
    FULL_DATA.get_or_init(|| {
        let json_str = fs::read_to_string("sample/bldg_risk.json").expect("データがない");
        serde_json::from_str(&json_str).expect("Failed to parse JSON")
    })
}

fn bench_zoom_out(c: &mut Criterion) {
    let data = get_full_data();

    let mut group_avg = c.benchmark_group("zoom_out_avg");
    group_avg.bench_function("z18", |b| {
        b.iter(|| {
            data.clone()
                .query()
                .zoom_out(18, Average)
                .raw_run()
                .unwrap();
        })
    });
    group_avg.finish();

    let mut group_sum = c.benchmark_group("zoom_out_sum");
    group_sum.bench_function("z18", |b| {
        b.iter(|| {
            data.clone()
                .query()
                .zoom_out(18, kasane_logic::merge_policy::Sum)
                .raw_run()
                .unwrap();
        })
    });
    group_sum.finish();

    let mut group_max = c.benchmark_group("zoom_out_max");
    group_max.bench_function("z18", |b| {
        b.iter(|| {
            data.clone()
                .query()
                .zoom_out(18, kasane_logic::merge_policy::Max)
                .raw_run()
                .unwrap();
        })
    });
    group_max.finish();
}

criterion_group!(benches, bench_zoom_out);
criterion_main!(benches);
