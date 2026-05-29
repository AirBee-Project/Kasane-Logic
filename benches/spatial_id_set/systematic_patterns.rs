//! SpatialIdSet の体系的なベンチマーク。6 種の空間パターンを網羅する。
//!
//! 各パターンは現実の空間データ分布をモデル化している：
//!
//! | パターン       | 説明                                              |
//! |----------------|---------------------------------------------------|
//! | Dense          | 連続した 3 次元ブロック（建物内部ボクセル）        |
//! | Sparse         | 全空間にランダム散在（分散センサー）               |
//! | Linear         | 対角線上のパス（道路・鉄道コリドー）              |
//! | MultiCluster   | 間隔を空けた 8 つの密なクラスター（都市ブロック） |
//! | Layered        | 一定間隔の水平スラブ（多層ビルのフロア）          |
//! | Checkerboard   | 交互配置ボクセル（ノードマージの最悪ケース）      |
//!
//! 各パターンに対してスイープする軸：
//!   ボクセル数 : 500 / 2_000 / 8_000
//!   操作種別   : Insert · Get · Remove · Union · Intersection · Difference
//!
//! ズームレベル効果を固定するため、全パターン z = 20 で計測する。
//! ズームスケーリンググループでは Dense パターンを使い z = 15〜22 を走査する。

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use kasane_logic::{SingleId, SpatialIdSet};
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::hint::black_box;

const Z: u8 = 20;
const COUNTS: &[usize] = &[500, 2_000, 8_000];

type PatternFn = fn(u8, usize) -> Vec<SingleId>;

const PATTERNS: &[(&str, PatternFn)] = &[
    ("Dense", dense_cluster),
    ("Sparse", sparse_random),
    ("Linear", linear_path),
    ("MultiCluster", multi_cluster),
    ("Layered", layered_floors),
    ("Checkerboard", checkerboard),
];

// ────────────────────────────────────────────────────────────────
// パターン生成関数
// ────────────────────────────────────────────────────────────────

/// 連続した 3 次元ブロック。FlexTree のノードマージが最大限発生するベストケース。
fn dense_cluster(z: u8, count: usize) -> Vec<SingleId> {
    let side = libm::ceil(libm::cbrt(count as f64)) as u32;
    let mut ids = Vec::with_capacity(count);
    'outer: for f in 0..side as i32 {
        for x in 0..side {
            for y in 0..side {
                if ids.len() >= count {
                    break 'outer;
                }
                ids.push(SingleId::new(z, f, x, y).unwrap());
            }
        }
    }
    ids
}

/// 全アドレス空間にランダム散在する ID。空間的局所性がほぼゼロ。
fn sparse_random(z: u8, count: usize) -> Vec<SingleId> {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let max_xy = (1u64 << z).saturating_sub(1) as u32;
    (0..count)
        .map(|_| {
            let x = rng.random_range(0..=max_xy);
            let y = rng.random_range(0..=max_xy);
            let f = rng.random_range(-500..500i32);
            SingleId::new(z, f, x, y).unwrap()
        })
        .collect()
}

/// 対角線上のボクセル列（f=0, x=i, y=i）。道路・コリドーをモデル化。
fn linear_path(z: u8, count: usize) -> Vec<SingleId> {
    (0..count)
        .map(|i| SingleId::new(z, 0, i as u32, i as u32).unwrap())
        .collect()
}

/// 4×2 グリッド上に配置した 8 つの密なクラスター。クラスター間には大きな隙間がある。
fn multi_cluster(z: u8, count: usize) -> Vec<SingleId> {
    const NUM: usize = 8;
    let per = count / NUM;
    let side = libm::ceil(libm::cbrt(per as f64)) as u32;
    let gap = side * 3;
    let mut ids = Vec::with_capacity(count);
    'outer: for c in 0..NUM {
        let base_x = (c as u32 % 4) * (side + gap);
        let base_y = (c as u32 / 4) * (side + gap);
        for f in 0..side as i32 {
            for x in base_x..base_x + side {
                for y in base_y..base_y + side {
                    if ids.len() >= count {
                        break 'outer;
                    }
                    ids.push(SingleId::new(z, f, x, y).unwrap());
                }
            }
        }
    }
    ids
}

/// 4 ボクセル間隔で積み重なった 10 枚の水平スラブ。多層ビルのフロアをモデル化。
fn layered_floors(z: u8, count: usize) -> Vec<SingleId> {
    const FLOORS: usize = 10;
    const GAP: i32 = 4;
    let per = count / FLOORS;
    let side = libm::ceil(libm::sqrt(per as f64)) as u32;
    let mut ids = Vec::with_capacity(count);
    'outer: for floor in 0..FLOORS {
        let f = floor as i32 * (1 + GAP);
        for x in 0..side {
            for y in 0..side {
                if ids.len() >= count {
                    break 'outer;
                }
                ids.push(SingleId::new(z, f, x, y).unwrap());
            }
        }
    }
    ids
}

/// 3 次元市松模様。隣接ボクセルが常に空白のためノードマージが一切発生しないワーストケース。
fn checkerboard(z: u8, count: usize) -> Vec<SingleId> {
    let side = libm::ceil(libm::cbrt((count * 2) as f64)) as u32;
    let mut ids = Vec::with_capacity(count);
    'outer: for f in 0..side as i32 {
        for x in 0..side {
            for y in 0..side {
                if (f as u32 + x + y).is_multiple_of(2) {
                    if ids.len() >= count {
                        break 'outer;
                    }
                    ids.push(SingleId::new(z, f, x, y).unwrap());
                }
            }
        }
    }
    ids
}

// ────────────────────────────────────────────────────────────────
// ヘルパー関数
// ────────────────────────────────────────────────────────────────

fn build_set(ids: &[SingleId]) -> SpatialIdSet {
    let mut set = SpatialIdSet::new();
    for id in ids {
        set.insert(id.clone());
    }
    set
}

/// 50% 要素重複を保証する 2 つのセットを生成する。
///
/// パターンから 2N 個の ID を生成し、以下のように分割する：
///   セット A = ids[0..N]
///   セット B = ids[N/2..3N/2]
///
/// 空間的に順序付けられたパターン（Dense・Linear・Layered・MultiCluster）では
/// 境界領域で実際の空間重複が発生する。Sparse（ランダム）では要素レベルで
/// 50% の重複があるが空間的な重複はほぼゼロになる。これは分散センサーデータの
/// 現実的な挙動と一致する。
fn make_pair(pattern_fn: PatternFn, z: u8, count: usize) -> (SpatialIdSet, SpatialIdSet) {
    let all = pattern_fn(z, count.saturating_mul(2));
    let n = all.len();
    let end_a = n.min(count);
    let start_b = end_a / 2;
    let end_b = (start_b + count).min(n);
    (build_set(&all[..end_a]), build_set(&all[start_b..end_b]))
}

// ────────────────────────────────────────────────────────────────
// ベンチマーク関数
// ────────────────────────────────────────────────────────────────

fn bench_insert(c: &mut Criterion) {
    for &(name, pattern_fn) in PATTERNS {
        let mut group = c.benchmark_group(format!("SpatialIdSet/Insert/{name}"));
        for &count in COUNTS {
            let ids = pattern_fn(Z, count);
            group.bench_with_input(BenchmarkId::from_parameter(count), &ids, |b, ids| {
                b.iter(|| black_box(build_set(ids)));
            });
        }
        group.finish();
    }
}

fn bench_get(c: &mut Criterion) {
    for &(name, pattern_fn) in PATTERNS {
        let mut group = c.benchmark_group(format!("SpatialIdSet/Get/{name}"));
        for &count in COUNTS {
            let ids = pattern_fn(Z, count);
            let set = build_set(&ids);
            group.bench_with_input(BenchmarkId::from_parameter(count), &ids, |b, ids| {
                b.iter(|| {
                    let mut total = 0usize;
                    for id in ids {
                        total += set.get(id).count();
                    }
                    black_box(total)
                });
            });
        }
        group.finish();
    }
}

fn bench_remove(c: &mut Criterion) {
    for &(name, pattern_fn) in PATTERNS {
        let mut group = c.benchmark_group(format!("SpatialIdSet/Remove/{name}"));
        for &count in COUNTS {
            let ids = pattern_fn(Z, count);
            group.bench_with_input(BenchmarkId::from_parameter(count), &ids, |b, ids| {
                // セットの構築はセットアップ（計測対象外）で行い、Remove のみを計測する
                b.iter_batched(
                    || build_set(ids),
                    |mut set| {
                        for id in ids {
                            let _ = set.remove(id);
                        }
                        black_box(set.is_empty())
                    },
                    BatchSize::SmallInput,
                );
            });
        }
        group.finish();
    }
}

fn bench_union(c: &mut Criterion) {
    for &(name, pattern_fn) in PATTERNS {
        let mut group = c.benchmark_group(format!("SpatialIdSet/Union/{name}"));
        for &count in COUNTS {
            let (a, b) = make_pair(pattern_fn, Z, count);
            group.bench_with_input(BenchmarkId::from_parameter(count), &count, |bench, _| {
                bench.iter(|| black_box(&a | &b));
            });
        }
        group.finish();
    }
}

fn bench_intersection(c: &mut Criterion) {
    for &(name, pattern_fn) in PATTERNS {
        let mut group = c.benchmark_group(format!("SpatialIdSet/Intersection/{name}"));
        for &count in COUNTS {
            let (a, b) = make_pair(pattern_fn, Z, count);
            group.bench_with_input(BenchmarkId::from_parameter(count), &count, |bench, _| {
                bench.iter(|| black_box(&a & &b));
            });
        }
        group.finish();
    }
}

fn bench_difference(c: &mut Criterion) {
    for &(name, pattern_fn) in PATTERNS {
        let mut group = c.benchmark_group(format!("SpatialIdSet/Difference/{name}"));
        for &count in COUNTS {
            let (a, b) = make_pair(pattern_fn, Z, count);
            group.bench_with_input(BenchmarkId::from_parameter(count), &count, |bench, _| {
                bench.iter(|| black_box(&a - &b));
            });
        }
        group.finish();
    }
}

/// z = 15〜22 を固定サイズ Dense パターンで走査し、ズームレベルの影響を単離する。
fn bench_zoom_scaling(c: &mut Criterion) {
    const FIXED_COUNT: usize = 1_000;
    let mut group = c.benchmark_group("SpatialIdSet/ZoomScaling/Dense");
    for z in [15u8, 18, 20, 22] {
        let ids = dense_cluster(z, FIXED_COUNT);
        group.bench_with_input(BenchmarkId::from_parameter(z), &ids, |b, ids| {
            b.iter(|| black_box(build_set(ids)));
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_insert,
    bench_get,
    bench_remove,
    bench_union,
    bench_intersection,
    bench_difference,
    bench_zoom_scaling,
);
criterion_main!(benches);
