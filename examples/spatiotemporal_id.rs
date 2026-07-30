//! 時空間 ID（仕様書 1.5.3 の `{z}/{f}/{x}/{y}_{i}/{t}`）の扱いを一通り示す例。
//!
//! ```bash
//! cargo run --example spatiotemporal_id
//! ```

use kasane_logic::{Interval, RangeId, SingleId, SpatialIdSet};

fn main() {
    // 仕様書 1.5.3 の例。時間間隔は任意の秒数を指定できる（1800 秒 = 30分）。
    let single_id = SingleId::new(12, 0, 3638, 1614)
        .unwrap()
        .with_time(1800, 809712)
        .unwrap();
    println!("SingleId      = {single_id}");
    println!("  interval    = {} 秒", single_id.interval().seconds());
    println!("  t           = {}", single_id.t());
    println!("  秒区間      = {:?}", single_id.seconds_range());

    // Unix 時刻から作ることもできる（t = floor(u / i) は内部で計算される）。
    let from_unix = SingleId::new(12, 0, 3638, 1614)
        .unwrap()
        .with_time_at(1800, 1_457_482_000)
        .unwrap();
    println!("Unix 時刻から = {from_unix}");
    assert_eq!(from_unix, single_id);

    // FlexTree のノードアドレス（FlexId）へは2の冪秒のセルへ分解して展開される。
    let cells: Vec<_> = single_id.clone().into_iter().collect();
    println!("FlexId        = {} 個へ分解", cells.len());
    for cell in &cells {
        println!(
            "                {cell}  ({} 秒幅)",
            cell.interval().seconds()
        );
    }

    // コレクションへ入れて取り出すと、隣接セルが結合されて元の {i}/{t} へ戻る。
    let mut set = SpatialIdSet::new();
    set.insert(single_id.clone());
    let restored: Vec<_> = set.flat_single_ids().collect();
    println!("木を経由      = {}", restored[0]);
    assert_eq!(restored[0], single_id);

    // 時間の「範囲」は RangeId が受け持つ（SingleId は単一セルのみ）。
    let range_id = RangeId::new(21, [10, 20], 10, 10)
        .unwrap()
        .with_time(Interval::MINUTE, [10, 20])
        .unwrap();
    println!("RangeId       = {range_id}");
    println!("  秒区間      = {:?}", range_id.seconds_range());
}
