use std::hint::black_box;

use kasane_logic::{Coordinate, Geometry, Line};
use memori::{Bench, Func};

fn main() {
    let mut long = Func::new("Line Function Long(Tokyo-Nagoya)")
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
        .add_function("Line Function Long distance(Tokyo-Nagoya)", |z| {
            let tokyo_station = Coordinate::new(35.681236, 139.767125, 0.0).unwrap();
            let nagoya_station = Coordinate::new(35.170915, 136.881537, 0.0).unwrap();

            let triangle = Line::new([tokyo_station, nagoya_station]);

            let iter = triangle
                .single_ids(*z as u8)
                .expect("Failed to get iterator");
            black_box(iter.count())
        });

    let mut short = Func::new("Line Function Short (Marunouchi South Exit-KITTE)")
        .add_bench(
            "Representative Value",
            "バージョンの変化で負荷が変化していないことを確認する",
            Bench::Instant(25),
        )
        .add_bench(
            "ZoomLevel Scaling",
            "ズームレベルが+1されると、最大でも8倍の負荷",
            Bench::Scaling(vec![15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25]),
        )
        .add_function("Line Function Short distance", |z| {
            let point_a = Coordinate::new(35.681000, 139.767000, 0.0).unwrap();
            let point_b = Coordinate::new(35.681200, 139.767200, 10.0).unwrap();

            let triangle = Line::new([point_a, point_b]);

            let iter = triangle
                .single_ids(*z as u8)
                .expect("Failed to get iterator");
            black_box(iter.count())
        });

    long.run_and_save().unwrap();
    short.run_and_save().unwrap();
}
