use crate::spatial_id::collection::flex_tree::core::FlexTreeCore;
use alloc::string::{String, ToString};

use crate::{SingleId, Source, SpatialIdTable};

fn segment(x: u32) -> SingleId {
    SingleId::new(20, 0, x, 100).unwrap()
}

/// `(x, 値)` の対応表へ畳み込む。
fn rows<V: Clone + Ord + Send + Sync + 'static>(
    table: &SpatialIdTable<V>,
) -> alloc::vec::Vec<(u32, V)> {
    let mut out: alloc::vec::Vec<(u32, V)> = table
        .iter()
        .flat_map(|(id, v)| id.single_ids().map(move |s| (s.x(), v.clone())))
        .collect();
    out.sort_by_key(|(x, _)| *x);
    out
}

fn int_table() -> SpatialIdTable<i32> {
    let mut t = SpatialIdTable::new();
    t.insert(segment(10), 1);
    t.insert(segment(11), 5);
    t.insert(segment(12), 10);
    t.insert(segment(13), 20);
    t
}

#[test]
fn filter_eq_keeps_only_that_value() {
    let out = int_table().query().filter_eq(10).raw_run_table().unwrap();

    assert_eq!(rows(&out), alloc::vec![(12, 10)]);
}

#[test]
fn filter_in_is_inclusive() {
    let out: SpatialIdTable<i32> = int_table()
        .query()
        .filter_in(5..=10)
        .raw_run_table()
        .unwrap();

    assert_eq!(rows(&out), alloc::vec![(11, 5), (12, 10)]);
}

#[test]
fn filter_in_open_bound() {
    let out: SpatialIdTable<i32> = int_table().query().filter_in(10..).raw_run_table().unwrap();

    assert_eq!(rows(&out), alloc::vec![(12, 10), (13, 20)]);
}

#[test]
fn filter_not_in_keeps_the_outside() {
    let out: SpatialIdTable<i32> = int_table()
        .query()
        .filter_not_in(5..=10)
        .raw_run_table()
        .unwrap();

    assert_eq!(rows(&out), alloc::vec![(10, 1), (13, 20)]);
}

/// 比較に必要なのは `Ord` だけなので、文字列でも同じように絞り込める。
#[test]
fn filter_values_works_for_text() {
    let mut t: SpatialIdTable<String> = SpatialIdTable::new();
    t.insert(segment(10), "apple".to_string());
    t.insert(segment(11), "banana".to_string());
    t.insert(segment(12), "cherry".to_string());

    let out: SpatialIdTable<String> = t
        .query()
        .filter_in("b".to_string()..="bz".to_string())
        .raw_run_table()
        .unwrap();

    assert_eq!(rows(&out), alloc::vec![(11, "banana".to_string())]);
}

/// 下限 > 上限 は実行前の検証で弾かれる。
#[test]
fn invalid_range_is_rejected_by_validate() {
    let result: Result<SpatialIdTable<i32>, _> = int_table()
        .query()
        .filter_in((
            core::ops::Bound::Included(100),
            core::ops::Bound::Included(1),
        ))
        .run()
        .map(Into::into);

    assert!(matches!(
        result,
        Err(crate::Error::InvalidQueryParameter(_))
    ));
}

/// 遅延評価（対象領域限定）でも同じ絞り込み結果になる。
#[test]
fn filter_values_via_lazy_view() {
    let query = int_table().query().filter_in(5..=10);

    let got: alloc::vec::Vec<i32> = query.lazy_get(segment(11)).unwrap().map(|(_, v)| v).collect();
    assert_eq!(got, alloc::vec![5]);

    // 範囲外の値だったSegmentは何も返らない。
    assert!(query.lazy_get(segment(13)).unwrap().next().is_none());
}

/// 切り分け: 範囲 (RangeId) を対象にした遅延取得で、複数Segmentが全て返ること。
#[test]
fn lazy_get_over_range_returns_all_segments() {
    use crate::RangeId;

    let mut t: SpatialIdTable<i32> = SpatialIdTable::new();
    for i in 0..4u32 {
        t.insert(SingleId::new(20, 0, 790000 + i, 500000).unwrap(), i as i32);
    }

    let bbox = RangeId::new(20, [0, 0], [790000, 790003], [500000, 500000]).unwrap();

    let query = t.query();
    let mut got: alloc::vec::Vec<i32> = query.lazy_get(bbox).unwrap().map(|(_, v)| v).collect();
    got.sort();

    assert_eq!(got, alloc::vec![0, 1, 2, 3]);
}

/// 刈った結果が正規形を保つこと。
///
/// `retain_values` は自前で `collapse_equal_children` を呼んで畳み直すため、
/// 「述語で消したあと左右の子が等価化した」ケースで正規形が崩れないことを固定する。
#[test]
fn filter_preserves_canonical_form() {
    use crate::SingleId;

    // 隣接4Segmentのうち2つを別値にして、フィルタで消すと残りが一様化する配置
    let mut core: FlexTreeCore<i32> = FlexTreeCore::new();
    for (x, y, v) in [(0, 0, 1), (1, 0, 1), (0, 1, 9), (1, 1, 9)] {
        core.insert(SingleId::new(4, 0, x, y).unwrap(), v);
    }
    core.assert_canonical();

    // 9 を消す → 残るのは (0,0) と (1,0) の値 1。
    // この2つは隣接する同値の兄弟なので、正規形では1つの異方Segmentへ畳まれる
    // （`count()` は葉=FlexId の数なので 2 ではなく 1 になる）。
    core.retain_values(|v| *v != 9);
    core.assert_canonical();
    assert_eq!(core.count(), 1, "隣接同値が畳まれていない");

    // 畳まれても覆っている空間は (0,0) と (1,0) の2Segment分であること
    let segments: alloc::vec::Vec<crate::SingleId> = core
        .iter()
        .flat_map(|(flex_id, _)| crate::RangeId::from(&flex_id).single_ids())
        .collect();
    let mut segments = segments;
    segments.sort();
    assert_eq!(
        segments,
        alloc::vec![
            SingleId::new(4, 0, 0, 0).unwrap(),
            SingleId::new(4, 0, 1, 0).unwrap()
        ]
    );

    // 全消し → 空になっても正規形
    core.retain_values(|_| false);
    core.assert_canonical();
    assert_eq!(core.count(), 0);
    assert!(core.is_empty());
}

/// 一様な部分木を丸ごと残す場合、`Arc` が共有されたまま（クローンされない）こと。
///
/// これが崩れると「変化していない部分木を作り直さない」という COW の効き目が失われる。
#[test]
fn filter_keeps_untouched_subtree_shared() {
    use crate::SingleId;

    let mut core: FlexTreeCore<i32> = FlexTreeCore::new();
    for x in 0..16u32 {
        core.insert(SingleId::new(4, 0, x, 0).unwrap(), 1);
    }
    let before = core.clone();

    // 全て述語を満たす → 木は一切変化しないはず
    core.retain_values(|v| *v == 1);

    assert!(
        core.root_ptr_eq(&before),
        "変化がないのに部分木が作り直されている（COW が効いていない）"
    );
    assert_eq!(core.count(), before.count());
}
