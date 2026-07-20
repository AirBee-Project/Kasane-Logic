use std::fs;
use std::time::Instant;

use kasane_logic::spatial_id::collection::query::merge_policy::{Average, Max};
use kasane_logic::{SpatialIdCollection, SpatialIdTable};

fn main() {
    let bldg_risk: SpatialIdTable<u32> =
        serde_json::from_str(&fs::read_to_string("sample/bldg_risk.json").unwrap()).unwrap();
    println!("Loaded bldg_risk.json");

    let base_query = || {
        bldg_risk
            .clone()
            .query()
            .zoom_out(22, Average)
            .extrude_f(25, 0, 5, Max)
    };

    println!("--- raw_run: 各順序での実行時間を計測 ---");
    // 1. X -> Y -> F
    let start = Instant::now();
    base_query()
        .falloff_linear_x(25, 3, Max)
        .falloff_linear_y(25, 3, Max)
        .falloff_linear_f(25, 2, Max)
        .fill_empty(0)
        .raw_run()
        .unwrap();
    println!("1. X(3) -> Y(3) -> F(2): {:?}", start.elapsed());

    // 2. X -> F -> Y
    let start = Instant::now();
    base_query()
        .falloff_linear_x(25, 3, Max)
        .falloff_linear_f(25, 2, Max)
        .falloff_linear_y(25, 3, Max)
        .fill_empty(0)
        .raw_run()
        .unwrap();
    println!("2. X(3) -> F(2) -> Y(3): {:?}", start.elapsed());

    // 3. Y -> X -> F
    let start = Instant::now();
    base_query()
        .falloff_linear_y(25, 3, Max)
        .falloff_linear_x(25, 3, Max)
        .falloff_linear_f(25, 2, Max)
        .fill_empty(0)
        .raw_run()
        .unwrap();
    println!("3. Y(3) -> X(3) -> F(2): {:?}", start.elapsed());

    // 4. Y -> F -> X
    let start = Instant::now();
    base_query()
        .falloff_linear_y(25, 3, Max)
        .falloff_linear_f(25, 2, Max)
        .falloff_linear_x(25, 3, Max)
        .fill_empty(0)
        .raw_run()
        .unwrap();
    println!("4. Y(3) -> F(2) -> X(3): {:?}", start.elapsed());

    // 5. F -> X -> Y  (This should be the optimized one, since F has radius 2, X has 3, Y has 3)
    let start = Instant::now();
    base_query()
        .falloff_linear_f(25, 2, Max)
        .falloff_linear_x(25, 3, Max)
        .falloff_linear_y(25, 3, Max)
        .fill_empty(0)
        .raw_run()
        .unwrap();
    println!(
        "5. F(2) -> X(3) -> Y(3) (Optimized order): {:?}",
        start.elapsed()
    );

    // 6. F -> Y -> X
    let start = Instant::now();
    base_query()
        .falloff_linear_f(25, 2, Max)
        .falloff_linear_y(25, 3, Max)
        .falloff_linear_x(25, 3, Max)
        .fill_empty(0)
        .raw_run()
        .unwrap();
    println!(
        "6. F(2) -> Y(3) -> X(3) (Optimized order): {:?}",
        start.elapsed()
    );

    println!("--- run: オプティマイザを通した実行 ---");
    let start = Instant::now();
    base_query()
        .falloff_linear_x(25, 3, Max)
        .falloff_linear_y(25, 3, Max)
        .falloff_linear_f(25, 2, Max)
        .fill_empty(0)
        .run()
        .unwrap();
    println!("run() の実行時間: {:?}", start.elapsed());
}
