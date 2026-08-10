use criterion::{Criterion, criterion_group, criterion_main};
use kasane_logic::{
    Source, SpatialIdTable,
    merge_policy::{Average, Max},
};
use std::fs;
use std::sync::OnceLock;

static FULL_DATA: OnceLock<SpatialIdTable<u32>> = OnceLock::new();

fn get_full_data() -> &'static SpatialIdTable<u32> {
    FULL_DATA.get_or_init(|| {
        let json_str = fs::read_to_string("sample/bldg_risk.json").expect("データがない");
        serde_json::from_str(&json_str).expect("Failed to parse JSON")
    })
}

fn bench_shift_and_falloff(c: &mut Criterion) {
    let data = get_full_data();
    c.bench_function("shift_and_falloff", |b| {
        b.iter(|| {
            data.clone()
                .query()
                .zoom_out(23, Average)
                .shift_x(24, 5)
                .shift_y(24, -5)
                .falloff_linear_x(24, 5, Max)
                .falloff_linear_y(24, 5, Max)
                .falloff_linear_f(24, 15, Max)
                .raw_run()
                .unwrap();
        })
    });
}

criterion_group!(benches, bench_shift_and_falloff);
criterion_main!(benches);
