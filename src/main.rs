use kasane_logic::SingleId;

fn main() {
    #[cfg(feature = "temporal_id")]
    temporal_demo();

    #[cfg(not(feature = "temporal_id"))]
    println!(
        "`temporal_id` feature が無効なため空間IDのみ: {}",
        SingleId::new(12, 0, 3638, 1614).unwrap()
    );
}

#[cfg(feature = "temporal_id")]
fn temporal_demo() {
    use kasane_logic::{RangeId, SpatialId, TemporalId};

    // 仕様書 1.5.3 の Spatio-temporal ID `{z}/{f}/{x}/{y}_{i}/{t}`。
    // 時間間隔は任意の秒数を指定できる（ここでは 1800 秒 = 30分）。
    let single_id = SingleId::new(12, 0, 3638, 1614)
        .unwrap()
        .with_temporal(TemporalId::new(1800, 809712).unwrap());
    println!("SingleId  = {single_id}");

    // FlexTree のノードアドレス（FlexId）へは、2の冪秒のセルへ分解されて展開される。
    let cells: Vec<_> = single_id.into_iter().collect();
    println!("FlexId    = {} 個へ分解", cells.len());
    for cell in &cells {
        println!("            {cell}");
    }

    // RangeId も同じ TemporalId を無条件に持てる。
    let range_id = RangeId::new(21, [10, 20], 10, 10)
        .unwrap()
        .with_temporal(TemporalId::new(60, [10, 20]).unwrap());
    println!("RangeId   = {range_id}");
    println!("temporal  = {}", range_id.temporal());
}
