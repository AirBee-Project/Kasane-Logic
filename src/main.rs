use kasane_logic::{RangeId, TableOnMemory};
use std::collections::HashMap;

fn main() {
    let mut table1: TableOnMemory<String> = TableOnMemory::new();

    // 1つ目のデータを挿入
    let id1 = RangeId::new(5, [9, 15], [10, 23], [10, 13]).unwrap();
    table1.insert(&id1, &"neko".to_string());

    // 2つ目のデータを挿入
    let id2 = RangeId::new(4, [3, 5], [9, 9], [3, 10]).unwrap();
    table1.insert(&id2, &"inu".to_string());

    // 値ごとに RangeId をまとめるための HashMap
    // Key: 値への参照 (&String), Value: RangeId のリスト (Vec<RangeId>)
    let mut grouped_map: HashMap<&String, Vec<RangeId>> = HashMap::new();

    // table1 から全ての (RangeId, &Value) を取り出してマップに格納
    for (range_id, val) in table1.range_ids() {
        grouped_map.entry(val).or_default().push(range_id);
    }

    // 値別に出力
    for (val, ids) in grouped_map {
        println!("Value: \"{}\"", val);
        for id in ids {
            println!("{},", id);
        }
    }
}
