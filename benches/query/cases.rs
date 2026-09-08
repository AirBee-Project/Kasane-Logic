//! Query ベンチマークのテストケース定義および共通ヘルパー。
//!
//! 全ての Query ベンチマーク（Criterion による速度測定および memory_usage による
//! CPUサイクル・メモリ測定）で同一のクエリ定義を一元管理する。

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

// ── Shift ──

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

pub fn shift_by_dim(table: SpatialIdTable<u32>, dim: &str, dist: i32) -> Query<u32> {
    match dim {
        "x" => shift_x(table, dist),
        "y" => shift_y(table, dist),
        "f" => shift_f(table, dist),
        "all" => shift_all(table, dist),
        _ => panic!("unknown dim: {}", dim),
    }
}

// ── Extrude ──

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

pub fn extrude_by_dim(table: SpatialIdTable<u32>, dim: &str, dist: i32) -> Query<u32> {
    match dim {
        "x" => extrude_x(table, dist),
        "y" => extrude_y(table, dist),
        "f" => extrude_f(table, dist),
        "all" => extrude_all(table, dist),
        _ => panic!("unknown dim: {}", dim),
    }
}

// ── Falloff ──

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

pub fn falloff_by_dim(table: SpatialIdTable<u32>, dim: &str, dist: u32) -> Query<u32> {
    match dim {
        "x" => falloff_x(table, dist),
        "y" => falloff_y(table, dist),
        "f" => falloff_f(table, dist),
        "all" => falloff_all(table, dist),
        _ => panic!("unknown dim: {}", dim),
    }
}

// ── ZoomOut ──

pub fn zoom_out(table: SpatialIdTable<u32>, target_z: u8) -> Query<u32> {
    let level = ZoomLevel::new(target_z).expect("valid zoom level");
    table.query().zoom_out(level, Average)
}

// ── FilterValues ──

pub fn filter_values(table: SpatialIdTable<u32>, threshold: u32) -> Query<u32> {
    let predicate = ValuePredicate::InRange(Bound::Included(threshold), Bound::Unbounded);
    table.query().filter_values(predicate)
}

// ── Workflow: RiskDiffusion ──

pub fn risk_diffusion(table: SpatialIdTable<u32>, radius: u32) -> Query<u32> {
    table
        .query()
        .zoom_out(22, Max)
        .falloff_f(25, radius, Some(Upper), FalloffPattern::Linear, Max)
        .falloff_x(25, radius, None, FalloffPattern::Linear, Max)
        .falloff_y(25, radius, None, FalloffPattern::Linear, Max)
}
