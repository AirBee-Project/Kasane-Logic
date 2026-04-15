use std::hint::black_box;

use kasane_logic::{Coordinate, CoverSingleIds, Sphere};
use memori::{Bench, Func, TrackingAllocator};

#[global_allocator]
static ALLOC: TrackingAllocator = TrackingAllocator;

fn main() {
    let mut func = Func::new("Sphere Function")
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
        .add_function("Sphere Function", |z| {
            let tokyo_station = Coordinate::new(35.681000, 139.767000, 0.0).unwrap();

            let triangle = Sphere::new(tokyo_station, 30.0);

            let iter = triangle
                .cover_single_ids(*z as u8)
                .expect("Failed to get iterator");
            black_box(iter.count())
        });

    func.run_and_save().unwrap();
}
