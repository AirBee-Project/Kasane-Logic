use crate::{F_MAX, FlexId, ShiftOps, SingleId, SpatialIdSet, SpatialIdTable};

fn table_with(z: u8, f: i32, x: u32, y: u32) -> SpatialIdTable<bool> {
    let mut table = SpatialIdTable::new();
    table.insert(SingleId::new(z, f, x, y).unwrap(), true);
    table
}

/// 指定したセル（z, f, x, y）に値が存在するか。
fn present(table: &SpatialIdTable<bool>, z: u8, f: i32, x: u32, y: u32) -> bool {
    let cell = SingleId::new(z, f, x, y).unwrap();
    table.get(&cell).next().is_some()
}

// ---- F方向（鉛直・巡回なし） ----

#[test]
fn shift_f_up_moves_cell() {
    let table = table_with(25, 0, 100, 100);
    let result = table.shift_f(25, 3).unwrap();

    assert!(!present(&result, 25, 0, 100, 100)); // 元の位置は空く
    assert!(present(&result, 25, 3, 100, 100)); // +3 へ移動
}

#[test]
fn shift_f_down_moves_cell() {
    let table = table_with(25, 10, 100, 100);
    let result = table.shift_f(25, -4).unwrap();

    assert!(!present(&result, 25, 10, 100, 100));
    assert!(present(&result, 25, 6, 100, 100));
}

#[test]
fn shift_f_out_of_range_is_error() {
    // z=25 の最上セルをさらに上へ動かすと範囲外。
    let table = table_with(25, F_MAX[25], 100, 100);
    assert!(table.shift_f(25, 1).is_err());
}

// ---- X方向（東西・巡回する） ----

#[test]
fn shift_x_moves_cell() {
    let table = table_with(25, 0, 100, 100);
    let result = table.shift_x(25, 5).unwrap();

    assert!(!present(&result, 25, 0, 100, 100));
    assert!(present(&result, 25, 0, 105, 100));
}

#[test]
fn shift_x_wraps_around_seam() {
    // z=2 の最東セル x=3 を +1 すると、経度の境界を越えて x=0 へ巡回する。
    let table = table_with(2, 0, 3, 0);
    let result = table.shift_x(2, 1).unwrap();

    assert!(!present(&result, 2, 0, 3, 0));
    assert!(present(&result, 2, 0, 0, 0));
}

#[test]
fn shift_x_full_circle_returns_to_origin() {
    // z=2 では一周が4セル。+4 で元の位置へ戻る。
    let table = table_with(2, 0, 1, 0);
    let result = table.shift_x(2, 4).unwrap();

    assert!(present(&result, 2, 0, 1, 0));
}

#[test]
fn shift_x_splits_when_crossing_seam() {
    // X方向のみ粗い（z1）セルは z2 換算で2セル幅。+1 でちょうど境界をまたぎ、
    // RangeIdの巡回表現と同様に z2 の2セル（x=3 と x=0）へ分割される。
    let mut table = SpatialIdTable::new();
    let id = FlexId::new(2, 0, 1, 1, 2, 0).unwrap(); // x だけ z1 / index1（= z2 の 2,3）
    table.insert(id, true);

    let result = table.shift_x(2, 1).unwrap();

    assert!(present(&result, 2, 0, 3, 0)); // 元の 3 が残り
    assert!(present(&result, 2, 0, 0, 0)); // 4 が巡回して 0 へ
    assert!(!present(&result, 2, 0, 2, 0)); // 元の 2 は空く
}

// ---- Y方向（南北・巡回なし） ----

#[test]
fn shift_y_moves_cell() {
    let table = table_with(25, 0, 100, 100);
    let result = table.shift_y(25, -3).unwrap();

    assert!(!present(&result, 25, 0, 100, 100));
    assert!(present(&result, 25, 0, 100, 97));
}

#[test]
fn shift_y_below_zero_is_error() {
    // y=0 を下方向へ動かすと範囲外。
    let table = table_with(25, 0, 100, 0);
    assert!(table.shift_y(25, -1).is_err());
}

#[test]
fn shift_y_above_max_is_error() {
    // z=2 の最北セル y=3 を上へ動かすと範囲外。
    let table = table_with(2, 0, 0, 3);
    assert!(table.shift_y(2, 1).is_err());
}

// ---- optimize による Shift の融合 ----

#[test]
fn optimize_fuses_shifts_in_all_dimensions() {
    use crate::SpatialIdCollection;
    use crate::spatial_id::collection::expr::plan::{Plan, UnaryOp};

    let optimized = table_with(25, 0, 100, 100)
        .plan()
        .shift_x(25, 5)
        .shift_y(25, -3)
        .shift_f(25, 3)
        .optimize();

    // 3 軸の Shift が 1 つの Shift ノードへ融合される。
    match &optimized {
        Plan::Unary(UnaryOp::Shift(_), inner) => {
            assert!(matches!(**inner, Plan::Source(_)));
        }
        _ => panic!("expected a single fused Shift node"),
    }

    // 融合後の結果は、逐次適用した結果と一致する。
    let fused = optimized.execution().unwrap();
    let sequential = table_with(25, 0, 100, 100)
        .shift_x(25, 5)
        .unwrap()
        .shift_y(25, -3)
        .unwrap()
        .shift_f(25, 3)
        .unwrap();

    assert!(present(&fused, 25, 3, 105, 97)); // x+5, y-3, f+3
    assert!(!present(&fused, 25, 0, 100, 100));
    assert!(present(&sequential, 25, 3, 105, 97));
    assert_eq!(fused.iter().count(), sequential.iter().count());
}

#[test]
fn optimize_keeps_separate_nodes_for_repeated_axis() {
    use crate::SpatialIdCollection;
    use crate::spatial_id::collection::expr::plan::{Plan, UnaryOp};

    let optimized = table_with(25, 0, 100, 100)
        .plan()
        .shift_x(25, 2)
        .shift_x(25, 3)
        .optimize();

    // 同じ X 軸が 2 回 → 1 つには融合できず、2 段の Shift ノードのまま残る。
    match &optimized {
        Plan::Unary(UnaryOp::Shift(_), inner) => {
            assert!(matches!(**inner, Plan::Unary(UnaryOp::Shift(_), _)));
        }
        _ => panic!("expected two separate shift nodes"),
    }

    // 結果は逐次適用（合計 +5）と一致する。
    let result = optimized.execution().unwrap();
    assert!(present(&result, 25, 0, 105, 100));
    assert!(!present(&result, 25, 0, 100, 100));
}

#[test]
fn optimize_drops_identity_shift() {
    use crate::SpatialIdCollection;
    use crate::spatial_id::collection::expr::plan::Plan;

    let optimized = table_with(25, 0, 100, 100)
        .plan()
        .shift_x(25, 0)
        .shift_f(25, 0)
        .optimize();

    // すべて移動量 0 → Shift ノードは消え、Source だけが残る。
    assert!(matches!(optimized, Plan::Source(_)));
}

/// map_cells の並列パス（rayon 有効 + 閾値 256 セル以上）を通す。
/// 並列でも挿入順が保たれ、件数・位置が逐次と一致することを確認する。
#[test]
fn shift_over_threshold_preserves_all_cells() {
    let z = 12;
    let mut set = SpatialIdSet::new();
    let mut expected = 0;
    // 偶数座標へ散らして配置し、隣接セルの併合を防ぐ（16 * 17 = 272 ≥ 256）。
    for x in 0..16u32 {
        for y in 0..17u32 {
            set.insert(SingleId::new(z, 0, x * 2, y * 2).unwrap());
            expected += 1;
        }
    }
    assert!(expected >= 256);

    let result = set.shift_f(z, 1).unwrap();

    // 件数は保存され、全セルが f=1 へ移動している。
    assert_eq!(result.iter().count(), expected);
    assert!(
        result
            .get(&SingleId::new(z, 1, 0, 0).unwrap())
            .next()
            .is_some()
    );
    assert!(
        result
            .get(&SingleId::new(z, 1, 30, 32).unwrap())
            .next()
            .is_some()
    );
    assert!(
        result
            .get(&SingleId::new(z, 0, 0, 0).unwrap())
            .next()
            .is_none()
    );
}

// ---- Set でも同じ演算が使える（総称化の確認） ----

#[test]
fn shift_works_on_set() {
    let mut set = SpatialIdSet::new();
    set.insert(SingleId::new(25, 0, 100, 100).unwrap());

    let result = set.shift_f(25, 3).unwrap();

    let moved = SingleId::new(25, 3, 100, 100).unwrap();
    let original = SingleId::new(25, 0, 100, 100).unwrap();
    assert!(result.get(&moved).next().is_some());
    assert!(result.get(&original).next().is_none());
}
