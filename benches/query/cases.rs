//! Query ベンチマークのテストケース定義および共通ヘルパー。
//!
//! 全ての Query ベンチマーク（Criterion による速度測定および memory_usage による
//! CPUサイクル・メモリ測定）で同一のケース配列を一元管理する。

#![allow(dead_code)]

use core::ops::Bound;
use kasane_logic::{
    RangeId, Side::Upper, Source, SpatialIdTable, ZoomLevel,
    merge_policy::{Average, Max},
    spatial_id::collection::query::{
        Query, ops::unary::falloff::FalloffPattern,
        ops::unary::filter_values::ValuePredicate,
    },
};

pub const DEFAULT_DISTANCES: [i32; 4] = [1, 5, 10, 15];
pub const ZOOM_LEVELS: [u8; 4] = [24, 22, 20, 18];
pub const FILTER_THRESHOLDS: [u32; 5] = [1, 2, 3, 4, 5];

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
            q.collect_table_within(target_fn(table)).unwrap().iter().count()
        } else {
            q.collect_table().unwrap().iter().count()
        }
    }
}

// クエリビルダー関数群

pub fn shift_x(table: SpatialIdTable<u32>, dist: i32) -> Query<u32> {
    table.query().shift_x(24, dist)
}

pub fn shift_y(table: SpatialIdTable<u32>, dist: i32) -> Query<u32> {
    table.query().shift_y(24, dist)
}

pub fn shift_f(table: SpatialIdTable<u32>, dist: i32) -> Query<u32> {
    table.query().shift_f(24, dist)
}

pub fn shift_all(table: SpatialIdTable<u32>, dist: i32) -> Query<u32> {
    table
        .query()
        .shift_x(24, dist)
        .shift_y(24, -dist)
        .shift_f(24, dist)
}

pub fn extrude_x(table: SpatialIdTable<u32>, dist: i32) -> Query<u32> {
    table.query().extrude_x(24, 0, dist as u32, Max)
}

pub fn extrude_y(table: SpatialIdTable<u32>, dist: i32) -> Query<u32> {
    table.query().extrude_y(24, 0, dist as u32, Max)
}

pub fn extrude_f(table: SpatialIdTable<u32>, dist: i32) -> Query<u32> {
    table.query().extrude_f(24, 0, dist, Max)
}

pub fn extrude_all(table: SpatialIdTable<u32>, dist: i32) -> Query<u32> {
    table
        .query()
        .extrude_x(24, 0, dist as u32, Max)
        .extrude_y(24, 0, dist as u32, Max)
        .extrude_f(24, 0, dist, Max)
}

pub fn falloff_x(table: SpatialIdTable<u32>, dist: u32) -> Query<u32> {
    table
        .query()
        .falloff_x(24, dist, None, FalloffPattern::Linear, Max)
}

pub fn falloff_y(table: SpatialIdTable<u32>, dist: u32) -> Query<u32> {
    table
        .query()
        .falloff_y(24, dist, None, FalloffPattern::Linear, Max)
}

pub fn falloff_f(table: SpatialIdTable<u32>, dist: u32) -> Query<u32> {
    table
        .query()
        .falloff_f(24, dist, None, FalloffPattern::Linear, Max)
}

pub fn falloff_all(table: SpatialIdTable<u32>, dist: u32) -> Query<u32> {
    table
        .query()
        .falloff_x(24, dist, None, FalloffPattern::Linear, Max)
        .falloff_y(24, dist, None, FalloffPattern::Linear, Max)
        .falloff_f(24, dist, None, FalloffPattern::Linear, Max)
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
    table
        .query()
        .zoom_out(22, Max)
        .falloff_f(25, radius, Some(Upper), FalloffPattern::Linear, Max)
        .falloff_x(25, radius, None, FalloffPattern::Linear, Max)
        .falloff_y(25, radius, None, FalloffPattern::Linear, Max)
}

// テストケース配列 (Table-Driven)


pub static SHIFT_CASES: &[TestCase] = &[
    TestCase { name: "shift_x/1", group: "Unary/Shift", scope: "Full", build: |t| shift_x(t, 1), target: None, test_collect: true },
    TestCase { name: "shift_x/5", group: "Unary/Shift", scope: "Full", build: |t| shift_x(t, 5), target: None, test_collect: true },
    TestCase { name: "shift_x/10", group: "Unary/Shift", scope: "Full", build: |t| shift_x(t, 10), target: None, test_collect: true },
    TestCase { name: "shift_x/15", group: "Unary/Shift", scope: "Full", build: |t| shift_x(t, 15), target: None, test_collect: true },
    TestCase { name: "shift_y/1", group: "Unary/Shift", scope: "Full", build: |t| shift_y(t, 1), target: None, test_collect: true },
    TestCase { name: "shift_y/5", group: "Unary/Shift", scope: "Full", build: |t| shift_y(t, 5), target: None, test_collect: true },
    TestCase { name: "shift_y/10", group: "Unary/Shift", scope: "Full", build: |t| shift_y(t, 10), target: None, test_collect: true },
    TestCase { name: "shift_y/15", group: "Unary/Shift", scope: "Full", build: |t| shift_y(t, 15), target: None, test_collect: true },
    TestCase { name: "shift_f/1", group: "Unary/Shift", scope: "Full", build: |t| shift_f(t, 1), target: None, test_collect: true },
    TestCase { name: "shift_f/5", group: "Unary/Shift", scope: "Full", build: |t| shift_f(t, 5), target: None, test_collect: true },
    TestCase { name: "shift_f/10", group: "Unary/Shift", scope: "Full", build: |t| shift_f(t, 10), target: None, test_collect: true },
    TestCase { name: "shift_f/15", group: "Unary/Shift", scope: "Full", build: |t| shift_f(t, 15), target: None, test_collect: true },
    TestCase { name: "shift_all/1", group: "Unary/Shift", scope: "Full", build: |t| shift_all(t, 1), target: None, test_collect: true },
    TestCase { name: "shift_all/5", group: "Unary/Shift", scope: "Full", build: |t| shift_all(t, 5), target: None, test_collect: true },
    TestCase { name: "shift_all/10", group: "Unary/Shift", scope: "Full", build: |t| shift_all(t, 10), target: None, test_collect: true },
    TestCase { name: "shift_all/15", group: "Unary/Shift", scope: "Full", build: |t| shift_all(t, 15), target: None, test_collect: true },
];

pub static EXTRUDE_CASES: &[TestCase] = &[
    TestCase { name: "extrude_x/1", group: "Unary/Extrude", scope: "Full", build: |t| extrude_x(t, 1), target: None, test_collect: true },
    TestCase { name: "extrude_x/5", group: "Unary/Extrude", scope: "Full", build: |t| extrude_x(t, 5), target: None, test_collect: true },
    TestCase { name: "extrude_x/10", group: "Unary/Extrude", scope: "Full", build: |t| extrude_x(t, 10), target: None, test_collect: true },
    TestCase { name: "extrude_x/15", group: "Unary/Extrude", scope: "Full", build: |t| extrude_x(t, 15), target: None, test_collect: true },
    TestCase { name: "extrude_y/1", group: "Unary/Extrude", scope: "Full", build: |t| extrude_y(t, 1), target: None, test_collect: true },
    TestCase { name: "extrude_y/5", group: "Unary/Extrude", scope: "Full", build: |t| extrude_y(t, 5), target: None, test_collect: true },
    TestCase { name: "extrude_y/10", group: "Unary/Extrude", scope: "Full", build: |t| extrude_y(t, 10), target: None, test_collect: true },
    TestCase { name: "extrude_y/15", group: "Unary/Extrude", scope: "Full", build: |t| extrude_y(t, 15), target: None, test_collect: true },
    TestCase { name: "extrude_f/1", group: "Unary/Extrude", scope: "Full", build: |t| extrude_f(t, 1), target: None, test_collect: true },
    TestCase { name: "extrude_f/5", group: "Unary/Extrude", scope: "Full", build: |t| extrude_f(t, 5), target: None, test_collect: true },
    TestCase { name: "extrude_f/10", group: "Unary/Extrude", scope: "Full", build: |t| extrude_f(t, 10), target: None, test_collect: true },
    TestCase { name: "extrude_f/15", group: "Unary/Extrude", scope: "Full", build: |t| extrude_f(t, 15), target: None, test_collect: true },
    TestCase { name: "extrude_all/1", group: "Unary/Extrude", scope: "Full", build: |t| extrude_all(t, 1), target: None, test_collect: true },
    TestCase { name: "extrude_all/5", group: "Unary/Extrude", scope: "Full", build: |t| extrude_all(t, 5), target: None, test_collect: true },
    TestCase { name: "extrude_all/10", group: "Unary/Extrude", scope: "Full", build: |t| extrude_all(t, 10), target: None, test_collect: true },
    TestCase { name: "extrude_all/15", group: "Unary/Extrude", scope: "Full", build: |t| extrude_all(t, 15), target: None, test_collect: true },
];

pub static FALLOFF_CASES: &[TestCase] = &[
    TestCase { name: "falloff_x/1", group: "Unary/Falloff", scope: "Full", build: |t| falloff_x(t, 1), target: None, test_collect: true },
    TestCase { name: "falloff_x/5", group: "Unary/Falloff", scope: "Full", build: |t| falloff_x(t, 5), target: None, test_collect: true },
    TestCase { name: "falloff_x/10", group: "Unary/Falloff", scope: "Full", build: |t| falloff_x(t, 10), target: None, test_collect: true },
    TestCase { name: "falloff_x/15", group: "Unary/Falloff", scope: "Full", build: |t| falloff_x(t, 15), target: None, test_collect: true },
    TestCase { name: "falloff_y/1", group: "Unary/Falloff", scope: "Full", build: |t| falloff_y(t, 1), target: None, test_collect: true },
    TestCase { name: "falloff_y/5", group: "Unary/Falloff", scope: "Full", build: |t| falloff_y(t, 5), target: None, test_collect: true },
    TestCase { name: "falloff_y/10", group: "Unary/Falloff", scope: "Full", build: |t| falloff_y(t, 10), target: None, test_collect: true },
    TestCase { name: "falloff_y/15", group: "Unary/Falloff", scope: "Full", build: |t| falloff_y(t, 15), target: None, test_collect: true },
    TestCase { name: "falloff_f/1", group: "Unary/Falloff", scope: "Full", build: |t| falloff_f(t, 1), target: None, test_collect: true },
    TestCase { name: "falloff_f/5", group: "Unary/Falloff", scope: "Full", build: |t| falloff_f(t, 5), target: None, test_collect: true },
    TestCase { name: "falloff_f/10", group: "Unary/Falloff", scope: "Full", build: |t| falloff_f(t, 10), target: None, test_collect: true },
    TestCase { name: "falloff_f/15", group: "Unary/Falloff", scope: "Full", build: |t| falloff_f(t, 15), target: None, test_collect: true },
    TestCase { name: "falloff_all/1", group: "Unary/Falloff", scope: "Full", build: |t| falloff_all(t, 1), target: None, test_collect: true },
    TestCase { name: "falloff_all/5", group: "Unary/Falloff", scope: "Full", build: |t| falloff_all(t, 5), target: None, test_collect: true },
    TestCase { name: "falloff_all/10", group: "Unary/Falloff", scope: "Full", build: |t| falloff_all(t, 10), target: None, test_collect: true },
    TestCase { name: "falloff_all/15", group: "Unary/Falloff", scope: "Full", build: |t| falloff_all(t, 15), target: None, test_collect: true },
];

pub static ZOOM_OUT_CASES: &[TestCase] = &[
    TestCase { name: "zoom_out_to/24", group: "Unary/ZoomOut", scope: "Full", build: |t| zoom_out(t, 24), target: None, test_collect: true },
    TestCase { name: "zoom_out_to/22", group: "Unary/ZoomOut", scope: "Full", build: |t| zoom_out(t, 22), target: None, test_collect: true },
    TestCase { name: "zoom_out_to/20", group: "Unary/ZoomOut", scope: "Full", build: |t| zoom_out(t, 20), target: None, test_collect: true },
    TestCase { name: "zoom_out_to/18", group: "Unary/ZoomOut", scope: "Full", build: |t| zoom_out(t, 18), target: None, test_collect: true },
];

pub static FILTER_CASES: &[TestCase] = &[
    TestCase { name: "greater_than_or_equal/1", group: "Unary/FilterValues", scope: "Full", build: |t| filter_values(t, 1), target: None, test_collect: true },
    TestCase { name: "greater_than_or_equal/2", group: "Unary/FilterValues", scope: "Full", build: |t| filter_values(t, 2), target: None, test_collect: true },
    TestCase { name: "greater_than_or_equal/3", group: "Unary/FilterValues", scope: "Full", build: |t| filter_values(t, 3), target: None, test_collect: true },
    TestCase { name: "greater_than_or_equal/4", group: "Unary/FilterValues", scope: "Full", build: |t| filter_values(t, 4), target: None, test_collect: true },
    TestCase { name: "greater_than_or_equal/5", group: "Unary/FilterValues", scope: "Full", build: |t| filter_values(t, 5), target: None, test_collect: true },
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
        test_collect: false, // 1億要素超のため Collect はOOM回避
    },
];

/// 総合パフォーマンス（CPU・メモリ・速度）測定用の代表ケース一覧
pub static CORE_BENCH_CASES: &[TestCase] = &[
    TestCase { name: "Falloff_X (dist=1)", group: "Unary/Falloff", scope: "Full", build: |t| falloff_x(t, 1), target: None, test_collect: true },
    TestCase { name: "Falloff_X (dist=5)", group: "Unary/Falloff", scope: "Full", build: |t| falloff_x(t, 5), target: None, test_collect: false },
    TestCase { name: "ZoomOut (z=20)", group: "Unary/ZoomOut", scope: "Full", build: |t| zoom_out(t, 20), target: None, test_collect: true },
    TestCase { name: "Shift_All (dist=5)", group: "Unary/Shift", scope: "Full", build: |t| shift_all(t, 5), target: None, test_collect: false },
    TestCase { name: "Extrude_X (dist=5)", group: "Unary/Extrude", scope: "Full", build: |t| extrude_x(t, 5), target: None, test_collect: false },
    TestCase {
        name: "RiskDiffusion (Region)",
        group: "Workflow/RiskDiffusion",
        scope: "Regional",
        build: |t| risk_diffusion(t, 5),
        target: Some(sample_region_target),
        test_collect: true,
    },
];
