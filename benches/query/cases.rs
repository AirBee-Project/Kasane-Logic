//! Query ベンチマークのテストケース定義および共通ヘルパー。
//!
//! 全ての Query ベンチマーク（Criterion による速度測定および memory_usage による
//! CPUサイクル・メモリ測定）で同一のケース配列を一元管理する。

#![allow(dead_code)]

use core::ops::Bound;
use kasane_logic::{
    RangeId,
    Side::Upper,
    Source, SpatialIdTable, ZoomLevel,
    merge_policy::{Average, Max},
    spatial_id::collection::query::{
        Query, ops::unary::falloff::FalloffPattern, ops::unary::filter_values::ValuePredicate,
    },
};

/// 代表的な局所領域（データが存在する領域から1つ選定して周辺±20マスを対象とする）
pub fn sample_region_target(table: &SpatialIdTable<u32>) -> RangeId {
    let sample_id = table.iter().next().map(|(id, _)| id).unwrap();
    let base_range = RangeId::from(sample_id);
    base_range
        .x_edges_shift(base_range.z(), -20, 20)
        .ok()
        .flatten()
        .and_then(|r| r.y_edges_shift(base_range.z(), -20, 20).ok().flatten())
        .unwrap_or(base_range)
}

// テストケース構造体

#[derive(Clone, Copy)]
pub struct TestCase {
    pub name: &'static str,
    pub group: &'static str,
    pub scope: &'static str,
    pub build: fn(SpatialIdTable<u32>) -> Query<u32>,
    pub target: Option<fn(&SpatialIdTable<u32>) -> RangeId>,
    /// Table への集約（Collect）をベンチマークで実行するか（巨大ワークフローではOOM防止のためfalse）
    pub test_collect: bool,
}

impl TestCase {
    /// クエリをストリーム（イテレータ）として評価し、要素数を返す。
    pub fn run_stream(&self, table: &SpatialIdTable<u32>) -> usize {
        let q = (self.build)(table.clone());
        if let Some(target_fn) = self.target {
            q.run_within(target_fn(table)).unwrap().count()
        } else {
            q.run().unwrap().count()
        }
    }

    /// クエリを評価して SpatialIdTable へ集約し、格納要素数を返す。
    pub fn run_collect(&self, table: &SpatialIdTable<u32>) -> usize {
        let q = (self.build)(table.clone());
        if let Some(target_fn) = self.target {
            q.collect_table_within(target_fn(table))
                .unwrap()
                .iter()
                .count()
        } else {
            q.collect_table().unwrap().iter().count()
        }
    }
}

// クエリビルダー関数群

// `sample/bldg_risk.json` の実データは z<=23 (z23が大半) までしか存在しない。
// これより細かい z を指定すると、shift/extrude/falloff の内部実装
// (`segment_scale = 1 << (max_z - item_z)`) が全アイテムを不要に分割してから
// 処理するため、指定距離から想定されるより大幅に重いワークロードになる。
// そのため実データの最大 z と揃えている。
const NATIVE_Z: u8 = 23;

pub fn shift_x(table: SpatialIdTable<u32>, dist: i32) -> Query<u32> {
    table.query().shift_x(NATIVE_Z, dist)
}

pub fn shift_y(table: SpatialIdTable<u32>, dist: i32) -> Query<u32> {
    table.query().shift_y(NATIVE_Z, dist)
}

pub fn shift_f(table: SpatialIdTable<u32>, dist: i32) -> Query<u32> {
    table.query().shift_f(NATIVE_Z, dist)
}

pub fn shift_all(table: SpatialIdTable<u32>, dist: i32) -> Query<u32> {
    table
        .query()
        .shift_x(NATIVE_Z, dist)
        .shift_y(NATIVE_Z, -dist)
        .shift_f(NATIVE_Z, dist)
}

pub fn extrude_x(table: SpatialIdTable<u32>, dist: i32) -> Query<u32> {
    // `dist as u32` は負値だとラップして XOutOfRange を引き起こすため、
    // 呼び出し側は必ず非負の距離を渡すこと(shift系のような符号付きオフセットではない)。
    debug_assert!(dist >= 0, "extrude distance must be non-negative");
    table.query().extrude_x(NATIVE_Z, 0, dist as u32, Max)
}

pub fn extrude_y(table: SpatialIdTable<u32>, dist: i32) -> Query<u32> {
    debug_assert!(dist >= 0, "extrude distance must be non-negative");
    table.query().extrude_y(NATIVE_Z, 0, dist as u32, Max)
}

pub fn extrude_f(table: SpatialIdTable<u32>, dist: i32) -> Query<u32> {
    table.query().extrude_f(NATIVE_Z, 0, dist, Max)
}

pub fn extrude_all(table: SpatialIdTable<u32>, dist: i32) -> Query<u32> {
    debug_assert!(dist >= 0, "extrude distance must be non-negative");
    table
        .query()
        .extrude_x(NATIVE_Z, 0, dist as u32, Max)
        .extrude_y(NATIVE_Z, 0, dist as u32, Max)
        .extrude_f(NATIVE_Z, 0, dist, Max)
}

pub fn falloff_x(table: SpatialIdTable<u32>, dist: u32) -> Query<u32> {
    table
        .query()
        .falloff_x(NATIVE_Z, dist, None, FalloffPattern::Linear, Max)
}

pub fn falloff_y(table: SpatialIdTable<u32>, dist: u32) -> Query<u32> {
    table
        .query()
        .falloff_y(NATIVE_Z, dist, None, FalloffPattern::Linear, Max)
}

pub fn falloff_f(table: SpatialIdTable<u32>, dist: u32) -> Query<u32> {
    table
        .query()
        .falloff_f(NATIVE_Z, dist, None, FalloffPattern::Linear, Max)
}

pub fn falloff_all(table: SpatialIdTable<u32>, dist: u32) -> Query<u32> {
    table
        .query()
        .falloff_x(NATIVE_Z, dist, None, FalloffPattern::Linear, Max)
        .falloff_y(NATIVE_Z, dist, None, FalloffPattern::Linear, Max)
        .falloff_f(NATIVE_Z, dist, None, FalloffPattern::Linear, Max)
}

pub fn zoom_out(table: SpatialIdTable<u32>, target_z: u8) -> Query<u32> {
    let level = ZoomLevel::new(target_z).expect("valid zoom level");
    table.query().zoom_out(level, Average)
}

pub fn filter_values(table: SpatialIdTable<u32>, threshold: u32) -> Query<u32> {
    let predicate = ValuePredicate::InRange(Bound::Included(threshold), Bound::Unbounded);
    table.query().filter_values(predicate)
}

pub fn risk_diffusion(table: SpatialIdTable<u32>, radius: u32) -> Query<u32> {
    // zoom_out 後のデータは z=22 に統一される。falloff の z をこれより細かく
    // 指定すると (前述の NATIVE_Z と同じ理由で) 全アイテムが不要に分割されて
    // 出力要素数が数百倍に膨れ上がるため、z=22 に揃える。
    const DIFFUSION_Z: u8 = 22;
    table
        .query()
        .zoom_out(DIFFUSION_Z, Max)
        .falloff_f(
            DIFFUSION_Z,
            radius,
            Some(Upper),
            FalloffPattern::Linear,
            Max,
        )
        .falloff_x(DIFFUSION_Z, radius, None, FalloffPattern::Linear, Max)
        .falloff_y(DIFFUSION_Z, radius, None, FalloffPattern::Linear, Max)
}

// テストケース配列 (Table-Driven)

pub static SHIFT_CASES: &[TestCase] = &[
    TestCase {
        name: "shift_x/1",
        group: "Unary/Shift",
        scope: "Full",
        build: |t| shift_x(t, 1),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "shift_x/5",
        group: "Unary/Shift",
        scope: "Full",
        build: |t| shift_x(t, 5),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "shift_x/10",
        group: "Unary/Shift",
        scope: "Full",
        build: |t| shift_x(t, 10),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "shift_x/15",
        group: "Unary/Shift",
        scope: "Full",
        build: |t| shift_x(t, 15),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "shift_y/1",
        group: "Unary/Shift",
        scope: "Full",
        build: |t| shift_y(t, 1),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "shift_y/5",
        group: "Unary/Shift",
        scope: "Full",
        build: |t| shift_y(t, 5),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "shift_y/10",
        group: "Unary/Shift",
        scope: "Full",
        build: |t| shift_y(t, 10),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "shift_y/15",
        group: "Unary/Shift",
        scope: "Full",
        build: |t| shift_y(t, 15),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "shift_f/1",
        group: "Unary/Shift",
        scope: "Full",
        build: |t| shift_f(t, 1),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "shift_f/5",
        group: "Unary/Shift",
        scope: "Full",
        build: |t| shift_f(t, 5),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "shift_f/10",
        group: "Unary/Shift",
        scope: "Full",
        build: |t| shift_f(t, 10),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "shift_f/15",
        group: "Unary/Shift",
        scope: "Full",
        build: |t| shift_f(t, 15),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "shift_all/1",
        group: "Unary/Shift",
        scope: "Full",
        build: |t| shift_all(t, 1),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "shift_all/5",
        group: "Unary/Shift",
        scope: "Full",
        build: |t| shift_all(t, 5),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "shift_all/10",
        group: "Unary/Shift",
        scope: "Full",
        build: |t| shift_all(t, 10),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "shift_all/15",
        group: "Unary/Shift",
        scope: "Full",
        build: |t| shift_all(t, 15),
        target: None,
        test_collect: true,
    },
];

pub static EXTRUDE_CASES: &[TestCase] = &[
    TestCase {
        name: "extrude_x/1",
        group: "Unary/Extrude",
        scope: "Full",
        build: |t| extrude_x(t, 1),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "extrude_x/5",
        group: "Unary/Extrude",
        scope: "Full",
        build: |t| extrude_x(t, 5),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "extrude_x/10",
        group: "Unary/Extrude",
        scope: "Full",
        build: |t| extrude_x(t, 10),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "extrude_x/15",
        group: "Unary/Extrude",
        scope: "Full",
        build: |t| extrude_x(t, 15),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "extrude_y/1",
        group: "Unary/Extrude",
        scope: "Full",
        build: |t| extrude_y(t, 1),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "extrude_y/5",
        group: "Unary/Extrude",
        scope: "Full",
        build: |t| extrude_y(t, 5),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "extrude_y/10",
        group: "Unary/Extrude",
        scope: "Full",
        build: |t| extrude_y(t, 10),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "extrude_y/15",
        group: "Unary/Extrude",
        scope: "Full",
        build: |t| extrude_y(t, 15),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "extrude_f/1",
        group: "Unary/Extrude",
        scope: "Full",
        build: |t| extrude_f(t, 1),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "extrude_f/5",
        group: "Unary/Extrude",
        scope: "Full",
        build: |t| extrude_f(t, 5),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "extrude_f/10",
        group: "Unary/Extrude",
        scope: "Full",
        build: |t| extrude_f(t, 10),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "extrude_f/15",
        group: "Unary/Extrude",
        scope: "Full",
        build: |t| extrude_f(t, 15),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "extrude_all/1",
        group: "Unary/Extrude",
        scope: "Full",
        build: |t| extrude_all(t, 1),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "extrude_all/5",
        group: "Unary/Extrude",
        scope: "Full",
        build: |t| extrude_all(t, 5),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "extrude_all/10",
        group: "Unary/Extrude",
        scope: "Full",
        build: |t| extrude_all(t, 10),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "extrude_all/15",
        group: "Unary/Extrude",
        scope: "Full",
        build: |t| extrude_all(t, 15),
        target: None,
        test_collect: true,
    },
];

pub static FALLOFF_CASES: &[TestCase] = &[
    TestCase {
        name: "falloff_x/1",
        group: "Unary/Falloff",
        scope: "Full",
        build: |t| falloff_x(t, 1),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "falloff_x/5",
        group: "Unary/Falloff",
        scope: "Full",
        build: |t| falloff_x(t, 5),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "falloff_x/10",
        group: "Unary/Falloff",
        scope: "Full",
        build: |t| falloff_x(t, 10),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "falloff_x/15",
        group: "Unary/Falloff",
        scope: "Full",
        build: |t| falloff_x(t, 15),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "falloff_y/1",
        group: "Unary/Falloff",
        scope: "Full",
        build: |t| falloff_y(t, 1),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "falloff_y/5",
        group: "Unary/Falloff",
        scope: "Full",
        build: |t| falloff_y(t, 5),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "falloff_y/10",
        group: "Unary/Falloff",
        scope: "Full",
        build: |t| falloff_y(t, 10),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "falloff_y/15",
        group: "Unary/Falloff",
        scope: "Full",
        build: |t| falloff_y(t, 15),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "falloff_f/1",
        group: "Unary/Falloff",
        scope: "Full",
        build: |t| falloff_f(t, 1),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "falloff_f/5",
        group: "Unary/Falloff",
        scope: "Full",
        build: |t| falloff_f(t, 5),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "falloff_f/10",
        group: "Unary/Falloff",
        scope: "Full",
        build: |t| falloff_f(t, 10),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "falloff_f/15",
        group: "Unary/Falloff",
        scope: "Full",
        build: |t| falloff_f(t, 15),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "falloff_all/1",
        group: "Unary/Falloff",
        scope: "Full",
        build: |t| falloff_all(t, 1),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "falloff_all/5",
        group: "Unary/Falloff",
        scope: "Full",
        build: |t| falloff_all(t, 5),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "falloff_all/10",
        group: "Unary/Falloff",
        scope: "Full",
        build: |t| falloff_all(t, 10),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "falloff_all/15",
        group: "Unary/Falloff",
        scope: "Full",
        build: |t| falloff_all(t, 15),
        target: None,
        test_collect: true,
    },
];

pub static ZOOM_OUT_CASES: &[TestCase] = &[
    TestCase {
        name: "zoom_out_to/22",
        group: "Unary/ZoomOut",
        scope: "Full",
        build: |t| zoom_out(t, 22),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "zoom_out_to/20",
        group: "Unary/ZoomOut",
        scope: "Full",
        build: |t| zoom_out(t, 20),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "zoom_out_to/18",
        group: "Unary/ZoomOut",
        scope: "Full",
        build: |t| zoom_out(t, 18),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "zoom_out_to/16",
        group: "Unary/ZoomOut",
        scope: "Full",
        build: |t| zoom_out(t, 16),
        target: None,
        test_collect: true,
    },
];

pub static FILTER_CASES: &[TestCase] = &[
    TestCase {
        name: "greater_than_or_equal/10",
        group: "Unary/FilterValues",
        scope: "Full",
        build: |t| filter_values(t, 10),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "greater_than_or_equal/30",
        group: "Unary/FilterValues",
        scope: "Full",
        build: |t| filter_values(t, 30),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "greater_than_or_equal/50",
        group: "Unary/FilterValues",
        scope: "Full",
        build: |t| filter_values(t, 50),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "greater_than_or_equal/70",
        group: "Unary/FilterValues",
        scope: "Full",
        build: |t| filter_values(t, 70),
        target: None,
        test_collect: true,
    },
    TestCase {
        name: "greater_than_or_equal/90",
        group: "Unary/FilterValues",
        scope: "Full",
        build: |t| filter_values(t, 90),
        target: None,
        test_collect: true,
    },
];

pub static WORKFLOW_CASES: &[TestCase] = &[
    TestCase {
        name: "RiskDiffusion (Regional)",
        group: "Workflow/RiskDiffusion",
        scope: "Regional",
        build: |t| risk_diffusion(t, 5),
        target: Some(sample_region_target),
        test_collect: true,
    },
    TestCase {
        name: "RiskDiffusion (Full)",
        group: "Workflow/RiskDiffusion",
        scope: "Full",
        build: |t| risk_diffusion(t, 10),
        target: None,
        // falloff の z を zoom_out 後のデータ解像度 (DIFFUSION_Z) に揃えたことで
        // 出力は約100万要素・ピークメモリ約860MBに収まることを実測済み（修正前は
        // falloff の z がデータより細かく、内部でアイテムが不要に分割された結果
        // 1億要素超に膨れ上がりOOM/ハングの危険があった）。
        test_collect: true,
    },
];

/// 総合パフォーマンス（CPU・メモリ・速度）測定用の代表ケース一覧。
/// 各ケースは SHIFT_CASES 等の該当エントリのコピーではなく、構造体更新構文で
/// そのエントリ自身を参照する。これにより元の配列側でパラメータを変更した際に
/// ここが古い値のまま取り残される(ドリフトする)ことがない。
pub fn core_bench_cases() -> Vec<TestCase> {
    // インデックス指定が配列の並び替えでずれていないことを検証する
    // (該当エントリが期待した dist/z のケースであることを名前で確認する)。
    debug_assert_eq!(FALLOFF_CASES[0].name, "falloff_x/1");
    debug_assert_eq!(FALLOFF_CASES[1].name, "falloff_x/5");
    debug_assert_eq!(ZOOM_OUT_CASES[1].name, "zoom_out_to/20");
    debug_assert_eq!(SHIFT_CASES[13].name, "shift_all/5");
    debug_assert_eq!(EXTRUDE_CASES[1].name, "extrude_x/5");
    debug_assert_eq!(WORKFLOW_CASES[0].name, "RiskDiffusion (Regional)");

    vec![
        TestCase {
            name: "Falloff_X (dist=1)",
            ..FALLOFF_CASES[0]
        },
        TestCase {
            name: "Falloff_X (dist=5)",
            test_collect: false,
            ..FALLOFF_CASES[1]
        },
        TestCase {
            name: "ZoomOut (z=20)",
            ..ZOOM_OUT_CASES[1]
        },
        TestCase {
            name: "Shift_All (dist=5)",
            test_collect: false,
            ..SHIFT_CASES[13]
        },
        TestCase {
            name: "Extrude_X (dist=5)",
            test_collect: false,
            ..EXTRUDE_CASES[1]
        },
        TestCase {
            name: "RiskDiffusion (Region)",
            ..WORKFLOW_CASES[0]
        },
    ]
}
