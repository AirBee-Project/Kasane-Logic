//! [`SpatialIdTable::new_in_shard`] の挙動テスト。
//!
//! 特に、完全に領域外の値を insert したときに値辞書へ孤児 rank を
//! 残さないことを確認する（子モジュールなので private フィールドを参照できる）。

#![cfg(test)]

use crate::{FlexId, SingleId, SpatialIdTable};

fn region(z: u8, f: i32, x: u32, y: u32) -> FlexId {
    FlexId::from(SingleId::new(z, f, x, y).unwrap())
}

#[test]
fn out_of_range_insert_leaves_no_orphan_rank() {
    let mut table = SpatialIdTable::<i32>::new_in_shard(region(2, 0, 0, 0));
    // 領域外（別タイル）への挿入。
    table.insert(SingleId::new(2, 0, 3, 3).unwrap(), 42);

    assert!(table.is_empty());
    assert_eq!(table.count(), 0);
    // 値辞書・rank が一切増えていない（rank が残らない）。
    assert!(table.dictionary.is_empty());
    assert!(table.reverse_dictionary.is_empty());
    assert_eq!(table.current_rank, 0);
}

#[test]
fn in_range_insert_allocates_rank() {
    let mut table = SpatialIdTable::<i32>::new_in_shard(region(2, 0, 0, 0));
    table.insert(SingleId::new(4, 0, 1, 1).unwrap(), 7);

    assert_eq!(table.count(), 1);
    assert_eq!(table.dictionary.len(), 1);
    assert_eq!(table.current_rank, 1);

    let got: Vec<(FlexId, i32)> = table
        .get(&SingleId::new(4, 0, 1, 1).unwrap())
        .map(|(id, v)| (id, *v))
        .collect();
    assert_eq!(
        got,
        vec![(FlexId::from(SingleId::new(4, 0, 1, 1).unwrap()), 7)]
    );
}

#[test]
fn coarse_insert_is_clipped_to_region() {
    let shard = region(2, 0, 0, 0);
    let mut table = SpatialIdTable::<i32>::new_in_shard(shard.clone());
    // ズーム0（全空間）→ 領域へ切り詰め。
    table.insert(SingleId::new(0, 0, 0, 0).unwrap(), 5);

    assert_eq!(table.count(), 1);
    let got: Vec<(FlexId, i32)> = table.get(&shard).map(|(id, v)| (id, *v)).collect();
    assert_eq!(got, vec![(shard, 5)]);
}
