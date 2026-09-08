//! Query エンジンのヒープメモリ使用量ベンチマーク。
//!
//! カスタムアロケータで追跡窓を制御し、入力データのロードを除いた
//! クエリ評価中のヒープメモリ使用量（ピーク・最終）を計測する。
//!
//! 実行方法:
//!   cargo bench --bench query_memory

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicUsize, Ordering};

mod cases;
mod utils;

static TRACKING: AtomicBool = AtomicBool::new(false);
static CURRENT: AtomicIsize = AtomicIsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

struct TrackingAllocator;

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() && TRACKING.load(Ordering::Relaxed) {
            let after = CURRENT.fetch_add(layout.size() as isize, Ordering::Relaxed)
                + layout.size() as isize;
            if after > 0 {
                let after_u = after as usize;
                let mut peak = PEAK.load(Ordering::Relaxed);
                while after_u > peak {
                    match PEAK.compare_exchange_weak(
                        peak,
                        after_u,
                        Ordering::Relaxed,
                        Ordering::Relaxed,
                    ) {
                        Ok(_) => break,
                        Err(p) => peak = p,
                    }
                }
            }
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        if TRACKING.load(Ordering::Relaxed) {
            CURRENT.fetch_sub(layout.size() as isize, Ordering::Relaxed);
        }
    }
}

#[global_allocator]
static ALLOCATOR: TrackingAllocator = TrackingAllocator;

fn start_tracking() {
    CURRENT.store(0, Ordering::SeqCst);
    PEAK.store(0, Ordering::SeqCst);
    TRACKING.store(true, Ordering::SeqCst);
}

fn stop_tracking() -> (usize, usize) {
    TRACKING.store(false, Ordering::SeqCst);
    let current = CURRENT.load(Ordering::SeqCst).max(0) as usize;
    let peak = PEAK.load(Ordering::SeqCst);
    (current, peak)
}

#[cfg(windows)]
fn get_process_cycle_time() -> u64 {
    unsafe {
        unsafe extern "system" {
            fn GetCurrentProcess() -> isize;
            fn QueryProcessCycleTime(hProcess: isize, cycle_time: *mut u64) -> i32;
        }
        let mut cycles = 0u64;
        QueryProcessCycleTime(GetCurrentProcess(), &mut cycles);
        cycles
    }
}

#[cfg(windows)]
fn estimate_core_frequency_hz() -> f64 {
    use std::time::Instant;
    unsafe {
        unsafe extern "system" {
            fn GetCurrentThread() -> isize;
            fn QueryThreadCycleTime(hThread: isize, cycle_time: *mut u64) -> i32;
        }
        let mut c1 = 0u64;
        let mut c2 = 0u64;
        QueryThreadCycleTime(GetCurrentThread(), &mut c1);
        let t1 = Instant::now();
        // 約 3ms スピンしてスレッド消費サイクルと実時間を測る
        while t1.elapsed().as_micros() < 3000 {
            core::hint::spin_loop();
        }
        let elapsed = t1.elapsed().as_secs_f64();
        QueryThreadCycleTime(GetCurrentThread(), &mut c2);
        let cycles = c2.saturating_sub(c1);
        if elapsed > 0.0 && cycles > 0 {
            cycles as f64 / elapsed
        } else {
            3.0e9 // フォールバック: 3.0 GHz
        }
    }
}

#[cfg(not(windows))]
fn get_process_cycle_time() -> u64 {
    0
}
#[cfg(not(windows))]
fn estimate_core_frequency_hz() -> f64 {
    3.0e9
}

static CORE_FREQ_HZ: std::sync::OnceLock<f64> = std::sync::OnceLock::new();

fn get_core_freq() -> f64 {
    *CORE_FREQ_HZ.get_or_init(estimate_core_frequency_hz)
}

#[derive(Clone, Copy, Debug)]
struct PerfResult {
    output_count: usize,
    peak_bytes: usize,
    final_bytes: usize,
    wall_time: std::time::Duration,
    cycles: u64,
    cpu_time_ms: f64,
    parallelism: f64,
}

fn fmt_bytes(b: usize) -> String {
    if b >= 1024 * 1024 * 1024 {
        format!("{:.2} GB", b as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if b >= 1024 * 1024 {
        format!("{:.2} MB", b as f64 / (1024.0 * 1024.0))
    } else if b >= 1024 {
        format!("{:.1} KB", b as f64 / 1024.0)
    } else {
        format!("{} B", b)
    }
}

fn measure<F, R>(f: F) -> (PerfResult, R)
where
    F: FnOnce() -> (usize, R),
{
    let freq = get_core_freq();
    start_tracking();
    let start_cycles = get_process_cycle_time();
    let start_wall = std::time::Instant::now();

    let (output_count, res) = f();

    let wall_time = start_wall.elapsed();
    let end_cycles = get_process_cycle_time();
    let (final_bytes, peak_bytes) = stop_tracking();

    let cycles = end_cycles.saturating_sub(start_cycles);
    let cpu_time_secs = cycles as f64 / freq;
    let cpu_time_ms = cpu_time_secs * 1000.0;
    let wall_secs = wall_time.as_secs_f64();
    let parallelism = if wall_secs > 0.0 {
        cpu_time_secs / wall_secs
    } else {
        0.0
    };

    (
        PerfResult {
            output_count,
            peak_bytes,
            final_bytes,
            wall_time,
            cycles,
            cpu_time_ms,
            parallelism,
        },
        res,
    )
}

// クエリ実行

fn print_row(name: &str, scope: &str, mode: &str, res: &PerfResult) {
    let wall_ms = res.wall_time.as_secs_f64() * 1000.0;
    let mcycles = res.cycles as f64 / 1_000_000.0;
    println!(
        "| {} | {} | {} | {} | {:.2} ms | {:.2} ms | {:.1} M | {:.1}x | {} | {} |",
        name,
        scope,
        mode,
        res.output_count,
        wall_ms,
        res.cpu_time_ms,
        mcycles,
        res.parallelism,
        fmt_bytes(res.peak_bytes),
        fmt_bytes(res.final_bytes)
    );
}

fn main() {
    let table = utils::get_full_data();
    let input_count = table.iter().count();

    let logical_cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    let freq_ghz = get_core_freq() / 1.0e9;

    println!("# Query Engine 総合パフォーマンスベンチマーク (速度・CPU並列度・メモリ)");
    println!();
    println!(
        "- 入力データ: `sample/bldg_risk.json` ({} アイテム)",
        input_count
    );
    println!(
        "- 論理CPUコア数: {} コア (推定コア周波数: {:.2} GHz)",
        logical_cores, freq_ghz
    );
    println!("- 計測方式: `QueryProcessCycleTime` (CPU クロックサイクル精密積算)");
    println!();
    println!(
        "| クエリ名 | 評価スコープ | 実行モード | 出力要素数 | 実行時間 (Wall) | CPU時間 (換算) | CPUサイクル | 実効並列度 | ピークメモリ | 最終メモリ |"
    );
    println!("|:---|:---|:---|---:|---:|---:|---:|---:|---:|---:|");

    for case in cases::CORE_BENCH_CASES {
        // Stream モード
        let (res, _) = measure(|| (case.run_stream(table), ()));
        print_row(case.name, case.scope, "Stream (count)", &res);

        // Collect モード
        if case.test_collect {
            let (res, _) = measure(|| (case.run_collect(table), ()));
            print_row(case.name, case.scope, "Collect (Table)", &res);
        }
    }

    println!();
    println!("- **実行時間 (Wall)**: 実測所要時間（ミリ秒）");
    println!("- **CPU時間**: プロセスが消費した総CPU計算時間（ユーザー + カーネル）");
    println!(
        "- **実効並列度**: `CPU時間 ÷ 実行時間`（1.0xならシングルスレッド相当、10.0xなら平均10コア稼働）"
    );
    println!("- **ピークメモリ**: クエリ実行中に到達したヒープメモリの最大瞬間使用量");
    println!("- **最終メモリ**: 評価完了後も保持されるヒープメモリ");
}
