#![cfg(all(feature = "json", feature = "rayon"))]

use crate::{
    Source, SpatialIdTable,
    spatial_id::collection::query::merge_policy::{Average, Max},
};
use alloc::vec::Vec;
use proptest::prelude::*;
use std::sync::OnceLock;

static BLDG_RISK: OnceLock<SpatialIdTable<u32>> = OnceLock::new();

fn get_bldg_risk() -> &'static SpatialIdTable<u32> {
    BLDG_RISK.get_or_init(|| {
        serde_json::from_str(&std::fs::read_to_string("sample/bldg_risk.json").unwrap()).unwrap()
    })
}

/// `n` 件のサブセットを取り出す。フル読み込み（約5万葉）のまま実行すると1ケースだけで
/// 20秒近くかかり(debugビルド)、proptestで何度も回すと現実的な時間で終わらない。
/// ベンチ（`benches/query/workflow/bldg_risk.rs`）と同じ縮小方針を使う。
fn get_subset(n: usize) -> SpatialIdTable<u32> {
    let full = get_bldg_risk();
    let mut subset = SpatialIdTable::new();
    for (id, &val) in full.iter().take(n) {
        subset.insert(id, val);
    }
    subset
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 8,
        .. ProptestConfig::default()
    })]

    /// 実際のビルドデータ（の一部）に対して、`zoom_out → extrude_f → falloff_x/y/f` という
    /// 現実的なワークフローを組み、最適化の有無で結果が変わらないことを確認する。
    /// `raw_run`（無最適化・宣言順そのまま）と `run`（最適化込み）に加え、`run_on_subset`
    /// （`lazy_get` の実体。`&self` のまま `plan_order` で並べ替える経路）も同じ結果になるかを見る。
    ///
    /// z=25までの extrude / falloff を含む現実的なワークフローは debug ビルドでは
    /// 1ケースあたり数秒かかるため、他のASTテスト群（`proptest_query` /
    /// `boundary_proptest`）と違いこのテストは既定では実行しない（`#[ignore]`）。
    /// `cargo test -- --ignored` または CI で実行する。
    #[test]
    #[ignore]
    fn ast_optimization_preserves_semantics(
        subset_n in prop_oneof![Just(200usize), Just(800usize)],
        zoom in 20..=22u8,
        ext_f_start in 0..2i32,
        ext_f_end in 2..5i32,
        falloff_x_rad in 1..3u32,
        falloff_y_rad in 1..3u32,
        falloff_f_rad in 1..3u32,
    ) {
        let bldg_risk = get_subset(subset_n);
        if bldg_risk.is_empty() {
            return Ok(());
        }

        let build = |t: SpatialIdTable<u32>| {
            t.query()
                .zoom_out(zoom, Average)
                .extrude_f(25, ext_f_start, ext_f_end, Max)
                .falloff_linear_x(25, falloff_x_rad, Max)
                .falloff_linear_y(25, falloff_y_rad, Max)
                .falloff_linear_f(25, falloff_f_rad, Max)
        };

        let unoptimized_result = build(bldg_risk.clone()).raw_run_table().unwrap();
        let optimized_result = build(bldg_risk.clone()).run_table().unwrap();

        assert_eq!(
            unoptimized_result.flat_single_ids().collect::<Vec<_>>(),
            optimized_result.flat_single_ids().collect::<Vec<_>>(),
            "AST optimization broke semantics!"
        );

        // 遅延経路（&self のまま plan_order で並べ替える run_on_subset）も同じ結果になること。
        // 対象を「最適化前の出力全体を覆うbounding box」にすれば、`raw_run` の全件と一致するはず。
        if let Some(bbox) = unoptimized_result.bounding_box() {
            let lazy_query = build(bldg_risk.clone());
            let mut lazy_result: Vec<(crate::FlexId, u32)> =
                lazy_query.lazy_get(bbox).unwrap().collect();
            lazy_result.sort_unstable();

            let mut expected: Vec<(crate::FlexId, u32)> = unoptimized_result
                .iter()
                .map(|(id, &v)| (id, v))
                .collect();
            expected.sort_unstable();

            assert_eq!(lazy_result, expected, "lazy_get(run_on_subset) broke semantics!");
        }
    }
}
