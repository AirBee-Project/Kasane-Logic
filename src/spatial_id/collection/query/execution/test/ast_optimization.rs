#![cfg(all(feature = "json", feature = "rayon"))]

use crate::{
    SpatialIdCollection, SpatialIdTable,
    spatial_id::collection::query::merge_policy::{Average, Max},
};
use proptest::prelude::*;
use std::sync::OnceLock;

static BLDG_RISK: OnceLock<SpatialIdTable<u32>> = OnceLock::new();

fn get_bldg_risk() -> &'static SpatialIdTable<u32> {
    BLDG_RISK.get_or_init(|| {
        serde_json::from_str(&std::fs::read_to_string("sample/bldg_risk.json").unwrap()).unwrap()
    })
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 3,
        .. ProptestConfig::default()
    })]

    #[test]
    #[ignore]
    fn ast_optimization_preserves_semantics(
        zoom in 20..=22u8,
        ext_f_start in 0..2i32,
        ext_f_end in 2..5i32,
        falloff_x_rad in 1..3u32,
        falloff_y_rad in 1..3u32,
        falloff_f_rad in 1..3u32,
    ) {
        let bldg_risk = get_bldg_risk();

        let unoptimized_result = bldg_risk
            .clone()
            .query()
            .zoom_out(zoom, Average)
            .extrude_f(25, ext_f_start, ext_f_end, Max)
            .falloff_linear_x(25, falloff_x_rad, Max)
            .falloff_linear_y(25, falloff_y_rad, Max)
            .falloff_linear_f(25, falloff_f_rad, Max)
            .raw_run()
            .unwrap();

        let optimized_result = bldg_risk
            .clone()
            .query()
            .zoom_out(zoom, Average)
            .extrude_f(25, ext_f_start, ext_f_end, Max)
            .falloff_linear_x(25, falloff_x_rad, Max)
            .falloff_linear_y(25, falloff_y_rad, Max)
            .falloff_linear_f(25, falloff_f_rad, Max)
            .run()
            .unwrap();

        assert_eq!(unoptimized_result, optimized_result, "AST optimization broke semantics!");
    }
}
