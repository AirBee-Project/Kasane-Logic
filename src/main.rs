use std::collections::HashMap;

use kasane_logic::{SingleId, SpatioTemporalId};

fn main() {
    //SingleIdを作成
    let mut single_id = SingleId::new(5, 10, 13, 4).unwrap();

    //時間のデータを追加
    single_id.set_t([30, 40]);

    println!("{}", single_id);
}
