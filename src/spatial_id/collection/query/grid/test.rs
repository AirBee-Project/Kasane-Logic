//! グリッド経路が木経路と同じ結果を返すことの検証。
//!
//! 木経路は `UnaryOperator::run`（`map_rebuild` 系）そのものなので、両者を突き合わせれば
//! 高速化で意味が変わっていないことを固定できる。

use crate::spatial_id::collection::query::execution::run_unary_chain;
use crate::spatial_id::collection::query::grid::try_run_grid;
use crate::spatial_id::collection::query::ops::unary::falloff::{
    FalloffPattern, falloff_f::FalloffF, falloff_x::FalloffX, falloff_y::FalloffY,
};
use crate::spatial_id::collection::query::ops::unary::shift::{
    shift_f::ShiftF, shift_x::ShiftX, shift_y::ShiftY,
};
use crate::spatial_id::collection::query::traits::UnaryOperator;
use crate::spatial_id::collection::query::working::WorkingTree;
use crate::{FlexId, SingleId};
use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::merge_policy::{Max, Min, Sum};

type Ops = Vec<Box<dyn UnaryOperator<u32>>>;

fn tree_from(ids: &[(FlexId, u32)]) -> WorkingTree<u32> {
    ids.iter().cloned().collect()
}

/// 本番経路（[`run_unary_chain`] = グリッド＋フォールバック）が、木経路だけで
/// 実行した結果と一致することを確かめる。これが常に成り立つべき不変条件。
fn assert_same(tree: WorkingTree<u32>, ops: Ops) {
    let mut expected = tree.clone();
    let tree_result = (|| {
        for op in &ops {
            op.run(&mut expected)?;
        }
        Ok::<(), crate::Error>(())
    })();

    let order: Vec<&dyn crate::spatial_id::collection::query::traits::UnaryOperator<u32>> =
        ops.iter().map(|op| &**op).collect();
    let got = run_unary_chain(&order, tree);

    match (tree_result, got) {
        (Err(_), Err(_)) => {}
        (Ok(()), Ok(got)) => {
            let mut a: Vec<(FlexId, u32)> = got.into_iter().collect();
            let mut b: Vec<(FlexId, u32)> = expected.into_iter().collect();
            a.sort();
            b.sort();
            assert_eq!(a, b, "本番経路と木経路で結果が違う");
        }
        (t, g) => panic!(
            "エラーの有無が食い違う: 木経路 ok={} / 本番経路 ok={}",
            t.is_ok(),
            g.is_ok()
        ),
    }
}

/// [`assert_same`] に加えて、実際にグリッド経路が使われたことも確かめる。
///
/// フォールバックだけを検証して「一致した」と安心してしまわないための歯止め。
fn assert_same_via_grid(tree: WorkingTree<u32>, ops: Ops) {
    let grid_ops: Vec<_> = ops
        .iter()
        .map(|op| op.grid_op().expect("この演算はグリッド対応のはず"))
        .collect();
    assert!(
        try_run_grid(&tree, &grid_ops, u64::MAX).is_some(),
        "グリッド経路が使われるはずの条件なのにフォールバックした"
    );
    assert_same(tree, ops);
}

fn sample() -> WorkingTree<u32> {
    tree_from(&[
        (FlexId::new(8, 3, 8, 100, 8, 100).unwrap(), 40),
        (FlexId::new(8, 3, 8, 101, 8, 100).unwrap(), 80),
        (FlexId::new(8, 4, 8, 100, 8, 101).unwrap(), 20),
        // 粗い葉（細分が必要）
        (FlexId::new(6, 0, 6, 24, 6, 24).unwrap(), 60),
        // 負の F
        (FlexId::new(8, -2, 8, 100, 8, 102).unwrap(), 55),
    ])
}

#[test]
fn shift_matches_tree_path() {
    for (dx, dy, df) in [(3i32, 2i32, 1i32), (-4, -1, -2), (0, 0, 0), (17, 5, 3)] {
        let ops: Ops = alloc::vec![
            Box::new(ShiftX::new(8u8, dx).unwrap()),
            Box::new(ShiftY::new(8u8, dy).unwrap()),
            Box::new(ShiftF::new(8u8, df).unwrap()),
        ];
        assert_same_via_grid(sample(), ops);
    }
}

/// 演算のズームが木より粗い場合（グリッド側は stride > 1 になる）。
#[test]
fn shift_at_coarser_zoom_matches_tree_path() {
    let ops: Ops = alloc::vec![
        Box::new(ShiftX::new(6u8, 2).unwrap()),
        Box::new(ShiftY::new(5u8, 1).unwrap()),
    ];
    assert_same_via_grid(sample(), ops);
}

#[test]
fn falloff_matches_tree_path() {
    for r in [1u32, 2, 5] {
        let ops: Ops = alloc::vec![
            Box::new(FalloffX::<Max>::new(8u8, r, None, FalloffPattern::Linear).unwrap()),
            Box::new(FalloffY::<Max>::new(8u8, r, None, FalloffPattern::Linear).unwrap()),
            Box::new(FalloffF::<Max>::new(8u8, r, None, FalloffPattern::Linear).unwrap()),
        ];
        assert_same_via_grid(sample(), ops);
    }
}

#[test]
fn falloff_with_other_commutative_policies() {
    let ops: Ops = alloc::vec![
        Box::new(FalloffX::<Min>::new(8u8, 3, None, FalloffPattern::Linear).unwrap()),
        Box::new(FalloffY::<Min>::new(8u8, 2, None, FalloffPattern::Linear).unwrap()),
    ];
    assert_same_via_grid(sample(), ops);

    let ops: Ops = alloc::vec![
        Box::new(FalloffF::<Sum>::new(8u8, 2, None, FalloffPattern::Linear).unwrap()),
        Box::new(FalloffX::<Sum>::new(8u8, 2, None, FalloffPattern::Linear).unwrap()),
    ];
    assert_same_via_grid(sample(), ops);
}

/// 演算ズームが木より粗い falloff（stride > 1 の経路）。
#[test]
fn falloff_at_coarser_zoom_matches_tree_path() {
    let ops: Ops = alloc::vec![Box::new(
        FalloffX::<Max>::new(6u8, 2, None, FalloffPattern::Linear).unwrap()
    )];
    assert_same_via_grid(sample(), ops);
    let ops: Ops = alloc::vec![Box::new(
        FalloffF::<Max>::new(6u8, 3, None, FalloffPattern::Linear).unwrap()
    )];
    assert_same_via_grid(sample(), ops);
}

/// X 軸は巡回する。日付変更線をまたぐ移動・伝播でも一致すること。
#[test]
fn x_axis_wraps_like_tree_path() {
    let span = 1u32 << 5;
    let tree = tree_from(&[
        (FlexId::new(5, 0, 5, 0, 5, 3).unwrap(), 10),
        (FlexId::new(5, 0, 5, span - 1, 5, 3).unwrap(), 20),
        (FlexId::new(5, 0, 5, span - 2, 5, 4).unwrap(), 30),
    ]);
    assert_same_via_grid(
        tree.clone(),
        alloc::vec![Box::new(ShiftX::new(5u8, 3).unwrap()) as Box<dyn UnaryOperator<u32>>],
    );
    assert_same_via_grid(
        tree.clone(),
        alloc::vec![Box::new(ShiftX::new(5u8, -5).unwrap()) as Box<dyn UnaryOperator<u32>>],
    );
    assert_same_via_grid(
        tree,
        alloc::vec![
            Box::new(FalloffX::<Max>::new(5u8, 4, None, FalloffPattern::Linear).unwrap())
                as Box<dyn UnaryOperator<u32>>
        ],
    );
}

/// F / Y が範囲外へ出る移動は、木経路と同じくエラーになる。
#[test]
fn out_of_range_shift_errors_like_tree_path() {
    let tree = tree_from(&[(FlexId::new(4, 15, 4, 1, 4, 15).unwrap(), 7)]);
    assert_same(
        tree.clone(),
        alloc::vec![Box::new(ShiftF::new(4u8, 5).unwrap()) as Box<dyn UnaryOperator<u32>>],
    );
    assert_same(
        tree,
        alloc::vec![Box::new(ShiftY::new(4u8, 5).unwrap()) as Box<dyn UnaryOperator<u32>>],
    );
}

/// F / Y の伝播が境界からはみ出す場合は、はみ出した先だけが落ちる。
#[test]
fn falloff_clipped_at_boundary_like_tree_path() {
    let tree = tree_from(&[
        (FlexId::new(4, 15, 4, 1, 4, 15).unwrap(), 90),
        (FlexId::new(4, -16, 4, 1, 4, 0).unwrap(), 90),
    ]);
    assert_same(
        tree.clone(),
        alloc::vec![
            Box::new(FalloffF::<Max>::new(4u8, 4, None, FalloffPattern::Linear).unwrap())
                as Box<dyn UnaryOperator<u32>>
        ],
    );
    assert_same(
        tree,
        alloc::vec![
            Box::new(FalloffY::<Max>::new(4u8, 4, None, FalloffPattern::Linear).unwrap())
                as Box<dyn UnaryOperator<u32>>
        ],
    );
}

/// X の falloff（巡回するので整列し直す）の直後に F / Y を平行移動すると、
/// レーン順は保たれるが Morton 順は崩れる。崩れた並びのまま木を組むと壊れる。
#[test]
fn shift_after_wrapping_falloff_reorders_before_building() {
    let tree = tree_from(&[(FlexId::new(4, 0, 4, 0, 4, 0).unwrap(), 1)]);
    for (fd, yd) in [(1i32, 0i32), (0, 1), (-1, 0), (3, 2)] {
        assert_same_via_grid(
            tree.clone(),
            alloc::vec![
                Box::new(FalloffX::<Max>::new(5u8, 1, None, FalloffPattern::Linear).unwrap())
                    as Box<dyn UnaryOperator<u32>>,
                Box::new(ShiftF::new(5u8, fd).unwrap()),
                Box::new(ShiftY::new(5u8, yd).unwrap()),
            ],
        );
    }
}

#[test]
fn empty_tree() {
    let tree: WorkingTree<u32> = WorkingTree::new();
    assert_same_via_grid(
        tree,
        alloc::vec![
            Box::new(FalloffX::<Max>::new(8u8, 3, None, FalloffPattern::Linear).unwrap())
                as Box<dyn UnaryOperator<u32>>
        ],
    );
}

/// 予算を超える（粗すぎる葉を細かいズームの演算で扱う）ときは平坦化を諦める。
#[test]
fn refuses_when_over_budget() {
    let tree: WorkingTree<u32> = tree_from(&[(FlexId::new(0, 0, 0, 0, 0, 0).unwrap(), 1)]);
    let op = ShiftX::new(24u8, 1).unwrap();
    let ops = alloc::vec![UnaryOperator::<u32>::grid_op(&op).unwrap()];
    assert!(try_run_grid(&tree, &ops, 1000).is_none());
}

/// 無作為な木と演算列で、グリッド経路が木経路と一致することを確かめる。
mod property {
    use super::*;
    use crate::SpatialIdTable;
    use proptest::prelude::*;

    #[derive(Debug, Clone, Copy)]
    enum Op {
        ShiftX(i32),
        ShiftY(i32),
        ShiftF(i32),
        FalloffX(u32, u8),
        FalloffY(u32, u8),
        FalloffF(u32, u8),
    }

    /// 可換なポリシー（グリッド経路が対象とするもの）だけを選ぶ。
    fn build(op: Op, z: u8) -> Box<dyn UnaryOperator<u32>> {
        macro_rules! falloff {
            ($ty:ident, $r:expr, $p:expr) => {
                match $p {
                    0 => Box::new($ty::<Max>::new(z, $r, None, FalloffPattern::Linear).unwrap())
                        as Box<dyn UnaryOperator<u32>>,
                    1 => Box::new($ty::<Min>::new(z, $r, None, FalloffPattern::Linear).unwrap())
                        as Box<dyn UnaryOperator<u32>>,
                    _ => Box::new($ty::<Sum>::new(z, $r, None, FalloffPattern::Linear).unwrap())
                        as Box<dyn UnaryOperator<u32>>,
                }
            };
        }
        match op {
            Op::ShiftX(d) => Box::new(ShiftX::new(z, d).unwrap()),
            Op::ShiftY(d) => Box::new(ShiftY::new(z, d).unwrap()),
            Op::ShiftF(d) => Box::new(ShiftF::new(z, d).unwrap()),
            Op::FalloffX(r, p) => falloff!(FalloffX, r, p),
            Op::FalloffY(r, p) => falloff!(FalloffY, r, p),
            Op::FalloffF(r, p) => falloff!(FalloffF, r, p),
        }
    }

    fn arb_op() -> impl Strategy<Value = Op> {
        prop_oneof![
            (-4..=4i32).prop_map(Op::ShiftX),
            (-4..=4i32).prop_map(Op::ShiftY),
            (-4..=4i32).prop_map(Op::ShiftF),
            (0..=3u32, 0..=2u8).prop_map(|(r, p)| Op::FalloffX(r, p)),
            (0..=3u32, 0..=2u8).prop_map(|(r, p)| Op::FalloffY(r, p)),
            (0..=3u32, 0..=2u8).prop_map(|(r, p)| Op::FalloffF(r, p)),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 64, ..ProptestConfig::default() })]

        #[test]
        fn grid_path_matches_tree_path(
            z in 5..=7u8,
            // 葉のズームをばらけさせて、異方・粗い葉が混ざるようにする
            items in prop::collection::vec(
                (0..=2u8, -6..=6i32, 0..=20u32, 0..=20u32, 1..200u32), 1..12),
            ops in prop::collection::vec(arb_op(), 1..4),
        ) {
            let mut table: SpatialIdTable<u32> = SpatialIdTable::new();
            for (coarser, f, x, y, v) in items {
                let leaf_z = z - coarser.min(z);
                let span = 1u32 << leaf_z;
                if let Ok(id) = SingleId::new(leaf_z, f, x % span, y % span) {
                    table.insert(id, v);
                }
            }
            let tree: WorkingTree<u32> = table.iter().map(|(id, v)| (id, *v)).collect();
            let built: Ops = ops.into_iter().map(|op| build(op, z)).collect();
            assert_same(tree, built);
        }
    }
}

/// 疑似乱数で散らした木でも、木経路と一致する。
#[test]
fn random_trees_match_tree_path() {
    let mut state = 0x9E3779B97F4A7C15u64;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };

    for round in 0..12u32 {
        let z = 7 + (round % 3) as u8;
        let mut ids = Vec::new();
        for _ in 0..40 {
            // ズームをばらけさせて異方・粗い葉を混ぜる
            let leaf_z = z - (next() % 3) as u8;
            let leaf_span = (1u64 << leaf_z).min(64);
            let f = (next() % 16) as i32 - 8;
            let x = (next() % leaf_span) as u32;
            let y = (next() % leaf_span) as u32;
            if let Ok(id) = SingleId::new(leaf_z, f, x, y) {
                ids.extend(id.into_iter().map(|c| (c, (next() % 100) as u32 + 1)));
            }
        }
        let tree = tree_from(&ids);
        let r = 1 + (next() % 4) as u32;
        let ops: Ops = alloc::vec![
            Box::new(ShiftX::new(z, (next() % 7) as i32 - 3).unwrap()),
            Box::new(FalloffX::<Max>::new(z, r, None, FalloffPattern::Linear).unwrap()),
            Box::new(FalloffY::<Max>::new(z, r, None, FalloffPattern::Linear).unwrap()),
            Box::new(FalloffF::<Max>::new(z, r, None, FalloffPattern::Linear).unwrap()),
        ];
        assert_same(tree, ops);
    }
}
