use alloc::vec;
use alloc::vec::Vec;
use crate::spatial_id::collection::query::merge_policy::{Average, Max};
use crate::{SingleId, Source, SpatialIdTable, ZoomLevel};

#[test]
fn zoom_out_average_8_children() {
    let mut table = SpatialIdTable::<i32>::new();

    // Z=20 の同じ親（Z=19）に属する8つのボクSegmentを配置
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
    let out = table
        .query()
        .zoom_out(ZoomLevel::new(19).unwrap(), Average)
        .run()
        .unwrap();

    let parent = SingleId::new(19, 0, 0, 0).unwrap();
    let result = out.get(parent.clone()).next().unwrap().1;

    // (10+20+30+40+50+60+70+80) / 8 = 360 / 8 = 45
    assert_eq!(*result, 45);
}

#[test]
fn zoom_out_max_partial_children() {
    let mut table = SpatialIdTable::<i32>::new();

    // 親が同じ 3つのボクSegmentだけ配置
    let id_000 = SingleId::new(20, 0, 0, 0).unwrap();
    let id_011 = SingleId::new(20, 0, 1, 1).unwrap();
    let id_111 = SingleId::new(20, 1, 1, 1).unwrap();

    table.insert(id_000, 10);
    table.insert(id_011, 99);
    table.insert(id_111, 5);

    // ZoomOut to Z=19 using Max policy
    let out = table.query().zoom_out(19, Max).run().unwrap();

    let parent = SingleId::new(19, 0, 0, 0).unwrap();
    let result = out.get(parent.clone()).next().unwrap().1;

    // Max of {10, 99, 5} is 99
    assert_eq!(*result, 99);
}

#[test]
fn zoom_out_multiple_parents_commutative_and_non_commutative() {
    use crate::FlexId;
    use crate::spatial_id::collection::query::merge_policy::Sum;

    let mut table = SpatialIdTable::<i32>::new();
    // 親1: (z: 1, f: 0, x: 0, y: 0) に属する子
    table.insert(FlexId::new(2, 0, 2, 0, 2, 0).unwrap(), 10);
    table.insert(FlexId::new(2, 0, 2, 1, 2, 0).unwrap(), 20);

    // 親2: (z: 1, f: 0, x: 1, y: 1) に属する子
    table.insert(FlexId::new(2, 0, 2, 2, 2, 2).unwrap(), 30);
    table.insert(FlexId::new(2, 0, 2, 3, 2, 3).unwrap(), 40);

    let sum_out: Vec<_> = table
        .clone()
        .query()
        .zoom_out(1, Sum)
        .run()
        .unwrap()
        .iter()
        .map(|(id, v)| (id, *v))
        .collect();

    let avg_out: Vec<_> = table
        .query()
        .zoom_out(1, Average)
        .run()
        .unwrap()
        .iter()
        .map(|(id, v)| (id, *v))
        .collect();

    let p1 = FlexId::new(1, 0, 1, 0, 1, 0).unwrap();
    let p2 = FlexId::new(1, 0, 1, 1, 1, 1).unwrap();

    assert_eq!(sum_out, vec![(p1, 30), (p2, 70)]);
    assert_eq!(avg_out, vec![(p1, 15), (p2, 35)]);
}

#[test]
fn zoom_out_lazily_short_circuits() {
    use crate::FlexId;
    use crate::spatial_id::collection::query::ops::unary::zoom_out::ZoomOut;
    use crate::spatial_id::collection::query::traits::{QueryIter, UnaryOperator};

    struct PanicAfterFirst {
        count: usize,
    }

    impl Iterator for PanicAfterFirst {
        type Item = Result<(FlexId, i32), crate::Error>;
        fn next(&mut self) -> Option<Self::Item> {
            self.count += 1;
            match self.count {
                1 => Some(Ok((FlexId::new(2, 0, 2, 0, 2, 0).unwrap(), 10))),
                2 => Some(Ok((FlexId::new(2, 0, 2, 2, 2, 0).unwrap(), 20))),
                _ => panic!("lazy evaluation short-circuit failed"),
            }
        }
    }

    let op = ZoomOut::<i32, Max>::new(ZoomLevel::new(1).unwrap());
    let iter = QueryIter::from_ordered(PanicAfterFirst { count: 0 });
    let mut out = op.run(iter).unwrap();

    let first = out.next().unwrap().unwrap();
    assert_eq!(first.0, FlexId::new(1, 0, 1, 0, 1, 0).unwrap());
    assert_eq!(first.1, 10);
    // 最初の親を取得した段階でイテレータを drop
    drop(out);
}

#[test]
fn zoom_out_propagates_stream_error() {
    use crate::FlexId;
    use crate::spatial_id::collection::query::ops::unary::zoom_out::ZoomOut;
    use crate::spatial_id::collection::query::traits::{QueryIter, UnaryOperator};

    let items = [
        Ok((FlexId::new(2, 0, 2, 0, 2, 0).unwrap(), 1)),
        Err(crate::Error::SourceRead("stream error".into())),
    ];
    let iter = QueryIter::from_ordered(items.into_iter());
    let op = ZoomOut::<i32, Max>::new(ZoomLevel::new(1).unwrap());
    let mut out = op.run(iter).unwrap();

    let first = out.next().unwrap();
    // 最初の親の完了前にエラーが来た場合はエラーが直ちに返る
    assert!(first.is_err());
}
