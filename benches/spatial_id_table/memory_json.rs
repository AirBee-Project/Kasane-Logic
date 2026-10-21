//! `sample/bldg_risk.json` を1つずつ insert して作った SpatialIdTable のヒープ使用量。
//!
//! JSON の読み込みと要素の列挙は計測の外で行い、insert だけを数える。
//!
//!   cargo bench --bench spatial_id_table_memory_json

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};

use kasane_logic::{FlexId, SpatialIdTable};

static TRACKING: AtomicBool = AtomicBool::new(false);
static CURRENT: AtomicIsize = AtomicIsize::new(0);

struct TrackingAllocator;

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() && TRACKING.load(Ordering::Relaxed) {
            CURRENT.fetch_add(layout.size() as isize, Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if TRACKING.load(Ordering::Relaxed) {
            CURRENT.fetch_sub(layout.size() as isize, Ordering::Relaxed);
        }
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: TrackingAllocator = TrackingAllocator;

fn main() {
    let table: SpatialIdTable<u32> =
        serde_json::from_str(&std::fs::read_to_string("sample/bldg_risk.json").unwrap()).unwrap();
    let items: Vec<(FlexId, u32)> = table.iter().map(|(id, v)| (id, *v)).collect();
    drop(table);

    CURRENT.store(0, Ordering::SeqCst);
    TRACKING.store(true, Ordering::SeqCst);
    let mut built = SpatialIdTable::<u32>::new();
    for &(id, v) in &items {
        built.insert(id, v);
    }
    TRACKING.store(false, Ordering::SeqCst);
    let bytes = CURRENT.load(Ordering::SeqCst);
    println!(
        "bldg_risk.json: FlexId {} 個, ヒープ {bytes} B ({:.1} B/ID)",
        built.count(),
        bytes as f64 / built.count() as f64
    );
}
