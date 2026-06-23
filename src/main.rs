//! SpatialIdSet バックエンド比較用の簡易ベンチマーク。
//!
//! バックエンドはコンパイル時に切り替わるため、2 回実行して比較する:
//! ```text
//! cargo run --release                    # FlexTree（既定）
//! cargo run --release --features morton  # Morton order
//! ```
//! 起動時にどちらのバックエンドか、rayon の有無を表示する。

use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::time::{Duration, Instant};

use kasane_logic::{ConflictPolicy, SetOps, SingleId, SpatialIdSet, SpatialIdTable};

// ────────────────────────────────────────────────────────────────
// ヒープ使用量を測るための追跡アロケータ
// ────────────────────────────────────────────────────────────────

static TRACKING: AtomicBool = AtomicBool::new(false);
/// 追跡窓内の正味確保バイト数（解放で減る）。
static NET: AtomicIsize = AtomicIsize::new(0);

struct TrackingAlloc;

unsafe impl GlobalAlloc for TrackingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(layout) };
        if !p.is_null() && TRACKING.load(Ordering::Relaxed) {
            NET.fetch_add(layout.size() as isize, Ordering::Relaxed);
        }
        p
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if TRACKING.load(Ordering::Relaxed) {
            NET.fetch_sub(layout.size() as isize, Ordering::Relaxed);
        }
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOC: TrackingAlloc = TrackingAlloc;

/// `build` が確保し続けている正味バイト数（＝戻り値が保持する構造のヒープ量）を測る。
fn measure_mem<T>(build: impl FnOnce() -> T) -> (T, usize) {
    NET.store(0, Ordering::SeqCst);
    TRACKING.store(true, Ordering::SeqCst);
    let value = build();
    TRACKING.store(false, Ordering::SeqCst);
    let net = NET.load(Ordering::SeqCst).max(0) as usize;
    (value, net)
}

/// ベンチで使うズームレベル。
const Z: u8 = 20;
/// 各パターンで計測する要素数。
const COUNTS: &[usize] = &[2_000, 10_000, 40_000];
/// 1 計測あたりの試行回数（最小値を採用）。
const ITERS: usize = 5;

fn main() {
    let backend = if cfg!(feature = "morton") {
        "Morton order"
    } else {
        "FlexTree"
    };
    let rayon = if cfg!(feature = "rayon") { "on" } else { "off" };

    println!("============================================================");
    println!(" SpatialIdSet benchmark");
    println!("   backend : {backend}");
    println!("   rayon   : {rayon}");
    println!("   zoom    : {Z}");
    println!("   iters   : {ITERS} (min taken)");
    println!("============================================================");

    let patterns: &[(&str, PatternFn)] = &[
        ("Dense", dense_cluster),
        ("Sparse", sparse_random),
        ("Linear", linear_path),
        ("MultiCluster", multi_cluster),
        ("Layered", layered_floors),
        ("Checkerboard", checkerboard),
    ];

    for &(name, pattern_fn) in patterns {
        println!("\n■ Pattern = {name}");
        println!(
            "  {:>7} | {:>8} | {:>9} | {:>9} | {:>9} | {:>9} | {:>9} | {:>9}",
            "N", "stored", "insert", "get", "remove", "union", "intersct", "diff"
        );
        println!("  {}", "-".repeat(86));

        for &count in COUNTS {
            run_pattern(name, pattern_fn, count);
        }
    }

    println!("\n(注) `stored` は実際に保持されたセル数。FlexTree は領域マージで圧縮され、");
    println!("     Morton は単一解像度セルをそのまま保持するため差が出る（圧縮率の指標）。");

    bench_heatmap();
}

/// ヒートマップ（値フィールド）ワークロード。
///
/// 同じ密な立方体に、値のエントロピーだけを変えて格納する:
/// - `1 band`   : 全セル同値（占有マスク相当）。
/// - `N bands`  : 空間的に連続な K 段バンド（量子化ヒートマップ）。
/// - `Continuous`: 全セルが異なる値（生の連続値フィールド）。
///
/// FlexTree のノードマージは「隣接セルが同値」のときだけ効くため、量子化が粗いほど
/// 圧縮・演算が速くなり、連続値では破綻する。その様子を stored / mem / build / combine で観測する。
fn bench_heatmap() {
    const N: usize = 20_000;
    let side = libm::ceil(libm::cbrt(N as f64)) as u32;
    let ids = dense_cluster(Z, N);

    println!("\n■ Heatmap (SpatialIdTable<u64>, dense cube, N={N})");
    println!(
        "  {:>11} | {:>8} | {:>10} | {:>9} | {:>9} | {:>9}",
        "value", "stored", "mem", "build", "combine", "stored(c)"
    );
    println!("  {}", "-".repeat(72));

    // (ラベル, バンド数 0=連続)
    let modes: &[(&str, u64)] = &[
        ("1 band", 1),
        ("16 bands", 16),
        ("256 bands", 256),
        ("Continuous", 0),
    ];

    for &(label, bands) in modes {
        // 値割り当て: bands==0 は全セル一意、そうでなければ x 方向の K 段バンド。
        let value_of = |id: &SingleId| -> u64 {
            if bands == 0 {
                // 全軸に依存する一意値。
                ((id.f() + 1000) as u64) * (side as u64) * (side as u64)
                    + (id.x() as u64) * (side as u64)
                    + (id.y() as u64)
            } else {
                (id.x() as u64) * bands / (side as u64)
            }
        };

        // build（メモリ・時間）。
        let (table, mem) = measure_mem(|| {
            let mut t: SpatialIdTable<u64> = SpatialIdTable::new();
            for id in &ids {
                t.insert(id.clone(), value_of(id));
            }
            t
        });
        let stored = table.count();

        let build = time_min_setup(SpatialIdTable::<u64>::new, |mut t| {
            for id in &ids {
                t.insert(id.clone(), value_of(id));
            }
            black_box(t.is_empty());
        });

        // combine（2つのヒートマップを加算 = 演算子経路でのヒートマップ合成）。
        // B は同じ立方体で y 方向のバンド（A と異なる場）にする。
        let value_of_b = |id: &SingleId| -> u64 {
            if bands == 0 {
                ((id.y() + 7) as u64) * (side as u64) * (side as u64) + (id.x() as u64)
            } else {
                (id.y() as u64) * bands / (side as u64)
            }
        };
        let mut tb: SpatialIdTable<u64> = SpatialIdTable::new();
        for id in &ids {
            tb.insert(id.clone(), value_of_b(id));
        }

        let combined: SpatialIdTable<u64> = table
            .combine_with(&tb, |a: Option<&u64>, b: Option<&u64>| {
                Some(a.copied().unwrap_or(0) + b.copied().unwrap_or(0))
            })
            .unwrap();
        let stored_c = combined.count();

        let combine = time_min(|| {
            let r: SpatialIdTable<u64> = table
                .combine_with(&tb, |a: Option<&u64>, b: Option<&u64>| {
                    Some(a.copied().unwrap_or(0) + b.copied().unwrap_or(0))
                })
                .unwrap();
            black_box(r.is_empty());
        });

        println!(
            "  {:>11} | {:>8} | {:>10} | {:>9} | {:>9} | {:>9}",
            label,
            stored,
            fmt_bytes(mem),
            fmt_dur(build),
            fmt_dur(combine),
            stored_c,
        );
    }

    println!(
        "\n(注) FlexTree は隣接同値セルをマージするため、量子化が粗いほど stored/mem/演算が軽い。"
    );
    println!("     連続値ではマージが効かず Morton 同様 stored≈N まで膨らむ（圧縮の破綻）。");
}

/// バイト数を読みやすい単位へ。
fn fmt_bytes(b: usize) -> String {
    if b < 1024 {
        format!("{b}B")
    } else if b < 1024 * 1024 {
        format!("{:.1}KB", b as f64 / 1024.0)
    } else {
        format!("{:.2}MB", b as f64 / (1024.0 * 1024.0))
    }
}

fn run_pattern(_name: &str, pattern_fn: PatternFn, count: usize) {
    let ids = pattern_fn(Z, count);
    let (a_ids, b_ids) = make_pair(pattern_fn, Z, count);

    // 保持セル数（バックエンドの圧縮の効き具合）。
    let stored = {
        let mut s = SpatialIdSet::new();
        for id in &ids {
            s.insert(id.clone());
        }
        s.count()
    };

    // insert: 空集合から全件挿入。
    let insert = time_min_setup(SpatialIdSet::new, |mut s| {
        for id in &ids {
            s.insert(id.clone());
        }
        black_box(s.is_empty());
    });

    // get: 構築済み集合へ全件問い合わせ。
    let base = build_set(&ids);
    let get = time_min(|| {
        let mut hits = 0usize;
        for id in &ids {
            hits += base.get(id).count();
        }
        black_box(hits);
    });

    // remove: 構築済み集合から全件削除（構築は計測外）。
    let remove = time_min_setup(
        || build_set(&ids),
        |mut s| {
            for id in &ids {
                let _ = s.remove(id);
            }
            black_box(s.is_empty());
        },
    );

    // union / intersection / difference: 50% 重複ペアの演算子経路。
    let a = build_set(&a_ids);
    let b = build_set(&b_ids);
    let union = time_min(|| {
        black_box(&a | &b);
    });
    let intersection = time_min(|| {
        black_box(&a & &b);
    });
    let difference = time_min(|| {
        black_box(&a - &b);
    });

    // expr 経路の union も一応確認（演算子経路との差を見る）。
    let _expr_union = time_min(|| {
        black_box(a.union_with(&b, ConflictPolicy::Overwrite).unwrap());
    });

    println!(
        "  {:>7} | {:>8} | {:>9} | {:>9} | {:>9} | {:>9} | {:>9} | {:>9}",
        count,
        stored,
        fmt_dur(insert),
        fmt_dur(get),
        fmt_dur(remove),
        fmt_dur(union),
        fmt_dur(intersection),
        fmt_dur(difference),
    );
}

// ────────────────────────────────────────────────────────────────
// 計測ヘルパー
// ────────────────────────────────────────────────────────────────

/// セットアップを計測対象外にして、`op` の最小実行時間を返す。
fn time_min_setup<S>(mut setup: impl FnMut() -> S, mut op: impl FnMut(S)) -> Duration {
    let mut best = Duration::MAX;
    for _ in 0..ITERS {
        let state = setup();
        let t = Instant::now();
        op(state);
        best = best.min(t.elapsed());
    }
    best
}

/// `op` の最小実行時間を返す（セットアップ不要なもの向け）。
fn time_min(mut op: impl FnMut()) -> Duration {
    let mut best = Duration::MAX;
    for _ in 0..ITERS {
        let t = Instant::now();
        op();
        best = best.min(t.elapsed());
    }
    best
}

/// Duration を読みやすい単位（µs / ms）の文字列にする。
fn fmt_dur(d: Duration) -> String {
    let us = d.as_secs_f64() * 1_000_000.0;
    if us < 1000.0 {
        format!("{us:.1}us")
    } else {
        format!("{:.2}ms", us / 1000.0)
    }
}

fn build_set(ids: &[SingleId]) -> SpatialIdSet {
    let mut set = SpatialIdSet::new();
    for id in ids {
        set.insert(id.clone());
    }
    set
}

/// 50% 要素重複を保証する 2 セット分の ID 列を生成する。
fn make_pair(pattern_fn: PatternFn, z: u8, count: usize) -> (Vec<SingleId>, Vec<SingleId>) {
    let all = pattern_fn(z, count.saturating_mul(2));
    let n = all.len();
    let end_a = n.min(count);
    let start_b = end_a / 2;
    let end_b = (start_b + count).min(n);
    (all[..end_a].to_vec(), all[start_b..end_b].to_vec())
}

// ────────────────────────────────────────────────────────────────
// パターン生成（criterion ベンチの patterns.rs と同等。RNG は内蔵）
// ────────────────────────────────────────────────────────────────

type PatternFn = fn(u8, usize) -> Vec<SingleId>;

/// splitmix64 ベースの決定的 PRNG（外部依存を避けるため内蔵）。
struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed)
    }
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// `0..=max` の一様乱数。
    fn range_u32(&mut self, max: u32) -> u32 {
        if max == u32::MAX {
            self.next_u64() as u32
        } else {
            (self.next_u64() % (max as u64 + 1)) as u32
        }
    }
    /// `lo..=hi` の一様乱数（i32）。
    fn range_i32(&mut self, lo: i32, hi: i32) -> i32 {
        let span = (hi - lo) as u64 + 1;
        lo + (self.next_u64() % span) as i32
    }
}

/// 連続した 3-D ブロック。FlexTree のノードマージが最大限効くベストケース。
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

/// 全アドレス空間にランダム散在。空間的局所性ゼロ。
fn sparse_random(z: u8, count: usize) -> Vec<SingleId> {
    let mut rng = Rng::new(42);
    let max_xy = (1u64 << z).saturating_sub(1) as u32;
    (0..count)
        .map(|_| {
            let x = rng.range_u32(max_xy);
            let y = rng.range_u32(max_xy);
            let f = rng.range_i32(-500, 499);
            SingleId::new(z, f, x, y).unwrap()
        })
        .collect()
}

/// 対角線上のボクセル列。道路・コリドーをモデル化。
fn linear_path(z: u8, count: usize) -> Vec<SingleId> {
    (0..count)
        .map(|i| SingleId::new(z, 0, i as u32, i as u32).unwrap())
        .collect()
}

/// 4×2 グリッド上の 8 クラスター。クラスター間は大きな隙間で分離。
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

/// 4 ボクセル間隔の水平スラブ 10 枚。多層ビルのフロアをモデル化。
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

/// 3-D 市松模様。ノードマージが一切発生しないワーストケース。
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
