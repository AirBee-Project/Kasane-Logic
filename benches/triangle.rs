use std::hint::black_box;

use kasane_logic::{Coordinate, CoverSingleIds, Triangle};
use memori::{Bench, Func, TrackingAllocator};

#[global_allocator]
static ALLOC: TrackingAllocator = TrackingAllocator;

fn main() {
    let mut func = Func::new("Triangle Function")
        .add_bench(
            "Representative Value",
            "バージョンの変化で負荷が変化していないことを確認",
            Bench::Instant(25),
        )
        .add_bench(
            "ZoomLevel Scaling",
            "ズームレベルが+1されると、最大でも8倍の負荷",
            Bench::Scaling(vec![15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25]),
        )
        .add_function("Triangle Function", |z| {
            let tokyo_station = Coordinate::new(35.681000, 139.767000, 0.0).unwrap();
            let point_b = Coordinate::new(35.681200, 139.767200, 10.0).unwrap();
            let point_c = Coordinate::new(35.680700, 139.767050, 5.0).unwrap();

            let triangle = Triangle::new([tokyo_station, point_b, point_c]);

            let iter = triangle
                .cover_single_ids(*z as u8)
                .expect("Failed to get iterator");
            black_box(iter.count())
        });

    func.run_and_save().unwrap();
}
