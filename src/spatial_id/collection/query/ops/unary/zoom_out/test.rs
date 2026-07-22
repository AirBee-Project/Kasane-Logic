use crate::spatial_id::collection::query::merge_policy::{Average, Max};
use crate::{SingleId, Source, SpatialIdTable, ZoomLevel};

#[test]
fn zoom_out_average_8_children() {
    let mut table = SpatialIdTable::<i32>::new();

    // Z=20 の同じ親（Z=19）に属する8つのボクセルを配置
    // 親IDは (z: 19, f: 0, x: 0, y: 0) になるように、子IDの x, y は 0 または 1 にする
    let id_000 = SingleId::new(20, 0, 0, 0).unwrap();
    let id_001 = SingleId::new(20, 0, 0, 1).unwrap();
    let id_010 = SingleId::new(20, 0, 1, 0).unwrap();
    let id_011 = SingleId::new(20, 0, 1, 1).unwrap();
    let id_100 = SingleId::new(20, 1, 0, 0).unwrap();
    let id_101 = SingleId::new(20, 1, 0, 1).unwrap();
    let id_110 = SingleId::new(20, 1, 1, 0).unwrap();
    let id_111 = SingleId::new(20, 1, 1, 1).unwrap();

    table.insert(id_000, 10);
    table.insert(id_001, 20);
    table.insert(id_010, 30);
    table.insert(id_011, 40);
    table.insert(id_100, 50);
    table.insert(id_101, 60);
    table.insert(id_110, 70);
    table.insert(id_111, 80);

    // ZoomOut to Z=19 (8 children -> 1 parent)
    let out: SpatialIdTable<i32> = table
        .query()
        .zoom_out(ZoomLevel::new(19).unwrap(), Average)
        .raw_run_into()
        .unwrap();

    let parent = SingleId::new(19, 0, 0, 0).unwrap();
    let result = out.get(&parent).next().unwrap().1;

    // (10+20+30+40+50+60+70+80) / 8 = 360 / 8 = 45
    assert_eq!(*result, 45);
}

#[test]
fn zoom_out_max_partial_children() {
    let mut table = SpatialIdTable::<i32>::new();

    // 親が同じ 3つのボクセルだけ配置
    let id_000 = SingleId::new(20, 0, 0, 0).unwrap();
    let id_011 = SingleId::new(20, 0, 1, 1).unwrap();
    let id_111 = SingleId::new(20, 1, 1, 1).unwrap();

    table.insert(id_000, 10);
    table.insert(id_011, 99);
    table.insert(id_111, 5);

    // ZoomOut to Z=19 using Max policy
    let out: SpatialIdTable<i32> = table.query().zoom_out(19, Max).raw_run_into().unwrap();

    let parent = SingleId::new(19, 0, 0, 0).unwrap();
    let result = out.get(&parent).next().unwrap().1;

    // Max of {10, 99, 5} is 99
    assert_eq!(*result, 99);
}
