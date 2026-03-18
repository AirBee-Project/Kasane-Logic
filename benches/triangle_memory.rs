use divan::{AllocProfiler, Bencher};
use kasane_logic::{Coordinate, Geometry, Triangle};

// メモリ計測を有効化するプロファイラを登録
#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    divan::main();
}

// Z=15から25までを自動的にループして計測
#[divan::bench(args = 15..=25)]
fn big_triangle_scaling(bencher: Bencher, z: u8) {
    // 1. 東京駅付近の三角形をセットアップ（計測の外側）
    let tokyo_station = Coordinate::new(35.681000, 139.767000, 0.0).unwrap();
    let point_b = Coordinate::new(35.681200, 139.767200, 10.0).unwrap();
    let point_c = Coordinate::new(35.680700, 139.767050, 5.0).unwrap();
    let triangle = Triangle::new([tokyo_station, point_b, point_c]);

    // 2. 計測開始
    bencher.bench(|| {
        let iter = triangle.single_ids(z).expect("Failed to get iterator");
        divan::black_box(iter.count())
    });
}
