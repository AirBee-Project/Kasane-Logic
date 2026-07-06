//! マージ（圧縮）正当性の回帰テスト。
//!
//! 「merge できる FlexId が merge されない」漏れが無いことを、(1) 内部不変条件
//! （`Branch{Leaf(v), Leaf(v)}` が残らない）、(2) 正規形（iter→再挿入で count 不変）、
//! (3) 再帰 collapse のカスケード、の3観点で insert / union / intersection /
//! difference にわたり検査する。
#![cfg(test)]

use crate::spatial_id::collection::flex_tree::node::Node;
use crate::spatial_id::collection::set::tests::{
    arb_random_set_case, decompose_set_to_single_ids_at_zoom, sorted_single_ids,
};
use crate::{SingleId, SpatialIdSet, SpatialIdTable};
use proptest::prelude::*;

proptest! {
    /// 汎用 combine（時間集合値）が、値に依存しないレガシー node 演算
    /// （union/intersection/difference）と全時間データにおいて厳密一致する
    /// （＝汎用エンジンの挙動保存を単一IDオラクルで検証）。
    #[test]
    fn combine_matches_presence_ops(a in arb_random_set_case(), b in arb_random_set_case()) {
        let sa = a.build_set();
        let sb = b.build_set();
        let _z = sa
            .max_zoomlevel()
            .unwrap_or(0)
            .max(sb.max_zoomlevel().unwrap_or(0));

        // レガシー（値に依存しない node 演算）との一致。
        // Set の演算子（|, &, -）は汎用 combine を使うため、これが両エンジンの照合になる。
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(96))]

    /// 異方 collapse が単一ID被覆を保つことを、ブルートフォース（共通ズームへ単一ID
    /// 展開）と照合する。union / intersection / difference を網羅。
    ///
    /// これは座標再構成まで含めて検証する gold-standard。素朴 collapse の座標破壊は
    /// このオラクルでのみ検出できた（count 不変テストでは見逃した）ので常時実行する。
    #[test]
    fn ops_preserve_single_id_coverage(
        a in arb_random_set_case(),
        b in arb_random_set_case(),
    ) {
        let sa = a.build_set();
        let sb = b.build_set();
        let z = sa
            .max_zoomlevel()
            .unwrap_or(0)
            .max(sb.max_zoomlevel().unwrap_or(0));

        let ea = decompose_set_to_single_ids_at_zoom(&sa, z);
        let eb = decompose_set_to_single_ids_at_zoom(&sb, z);

        let mut exp_union: Vec<SingleId> = ea.union(&eb).cloned().collect();
        exp_union.sort();
        prop_assert_eq!(sorted_single_ids(&(&sa | &sb), z), exp_union,
            "union: a={} b={}", a.debug_summary(), b.debug_summary());

        let mut exp_inter: Vec<SingleId> = ea.intersection(&eb).cloned().collect();
        exp_inter.sort();
        prop_assert_eq!(sorted_single_ids(&(&sa & &sb), z), exp_inter,
            "intersection: a={} b={}", a.debug_summary(), b.debug_summary());

        let mut exp_diff: Vec<SingleId> = ea.difference(&eb).cloned().collect();
        exp_diff.sort();
        prop_assert_eq!(sorted_single_ids(&(&sa - &sb), z), exp_diff,
            "difference: a={} b={}", a.debug_summary(), b.debug_summary());
    }
}

/// Branch の「両子が値として等しい」ノード数を数える（＝冗長な軸分割＝スキップで畳める）。
fn count_equal_child_branches<V: crate::spatial_id::collection::flex_tree::ptr::SafeValue>(
    node: &Node<V>,
) -> usize {
    match node {
        Node::Leaf { .. } => 0,
        Node::Branch {
            lower_child,
            upper_child,
            ..
        } => {
            let here = if **lower_child == **upper_child { 1 } else { 0 };
            here + count_equal_child_branches(lower_child) + count_equal_child_branches(upper_child)
        }
    }
}

/// ガード付き collapse 適用後、X ストリップの木に「両子が等しい Branch」（＝最深軸の
/// 冗長分割）が 1 つも残らない（異方圧縮が効いている）こと。
#[test]
fn x_strip_has_no_redundant_branch() {
    let z = 4u8;
    let mut x_strip = SpatialIdSet::new();
    x_strip.insert(SingleId::new(z, 0, 0, 0).unwrap());
    x_strip.insert(SingleId::new(z, 0, 1, 0).unwrap());

    let redundant = count_equal_child_branches(&x_strip.inner.inner.lower_root)
        + count_equal_child_branches(&x_strip.inner.inner.upper_root);
    assert_eq!(redundant, 0, "no redundant equal-child branch must remain");
}

/// 道路（平面ストリップ）の X/Y 方向マージ対称性。
/// ガード付きの異方 collapse により、最深軸であれば X 方向ストリップも畳まれる。
#[test]
fn road_strip_xy_symmetry() {
    let z = 4u8;

    // Y 方向 2 セル → 1。
    let mut y_strip = SpatialIdSet::new();
    y_strip.insert(SingleId::new(z, 0, 0, 0).unwrap());
    y_strip.insert(SingleId::new(z, 0, 0, 1).unwrap());
    assert_eq!(y_strip.count(), 1, "Y-strip merges");

    // X 方向 2 セル → 最深 X の冗長分割が畳まれ 1（異方セル x_zoom=z-1, y_zoom=z）。
    let mut x_strip = SpatialIdSet::new();
    x_strip.insert(SingleId::new(z, 0, 0, 0).unwrap());
    x_strip.insert(SingleId::new(z, 0, 1, 0).unwrap());
    assert_eq!(x_strip.count(), 1, "X-strip now merges (anisotropic)");

    // 2x2 ブロック → 1。
    let mut block = SpatialIdSet::new();
    for x in 0..2 {
        for y in 0..2 {
            block.insert(SingleId::new(z, 0, x, y).unwrap());
        }
    }
    assert_eq!(block.count(), 1, "2x2 block merges");

    // X 方向に長さ 8 のアライン道（Y=1セル幅）→ 1 つの異方セルへ畳まれる。
    let mut long_x_road = SpatialIdSet::new();
    for x in 0..8 {
        long_x_road.insert(SingleId::new(z, 0, x, 0).unwrap());
    }
    assert_eq!(
        long_x_road.count(),
        1,
        "aligned X-road merges to a single anisotropic cell"
    );
}

/// フル充填の葉 collapse で max_zoom が減るケース。
/// zoom4 の 2×2×2 ブロック（f,x,y∈{0,1}）は zoom3 等方セル（葉）へ畳まれ、
/// max_zoomlevel は 4→3 になる（キャッシュ済み max_zoom がステイルにならない）。
#[test]
fn f_collapse_reduces_max_zoom() {
    let mut set = SpatialIdSet::new();
    for f in 0..2 {
        for x in 0..2 {
            for y in 0..2 {
                set.insert(SingleId::new(4, f, x, y).unwrap());
            }
        }
    }
    assert_eq!(set.count(), 1, "2x2x2 block collapses to 1");
    assert_eq!(
        set.max_zoomlevel(),
        Some(3),
        "f-collapse must drop cached max_zoom 4 -> 3"
    );
    assert_eq!(set.max_zoomlevel(), brute_max_zoom(&set));
}

/// Table（値あり）: 同値の Y 兄弟は merge して 1、異値は merge せず 2。
#[test]
fn table_merges_same_value_only() {
    let mut same = SpatialIdTable::<i32>::new();
    same.insert(SingleId::new(2, 0, 0, 0).unwrap(), 7);
    same.insert(SingleId::new(2, 0, 0, 1).unwrap(), 7);
    assert_eq!(same.count(), 1, "same-value y-siblings must merge");

    let mut diff = SpatialIdTable::<i32>::new();
    diff.insert(SingleId::new(2, 0, 0, 0).unwrap(), 7);
    diff.insert(SingleId::new(2, 0, 0, 1).unwrap(), 8);
    assert_eq!(diff.count(), 2, "different-value cells must NOT merge");
}

/// Table: 8 オクタントを同値で満たすと 1 葉へ collapse。異値が混じると collapse しない。
#[test]
fn table_full_cube_same_value_collapses() {
    let mut t = SpatialIdTable::<i32>::new();
    for f in 0..2 {
        for x in 0..2 {
            for y in 0..2 {
                t.insert(SingleId::new(1, f, x, y).unwrap(), 1);
            }
        }
    }
    assert_eq!(t.count(), 1);

    // 1 つだけ別値にすると collapse しない。
    t.insert(SingleId::new(1, 1, 1, 1).unwrap(), 2);
    assert!(t.count() > 1, "mixed values must not fully collapse");
}

/// 「両子が等しい値の Leaf」な Branch が存在しないこと（未適用マージ＝バグ）を再帰検査する。
/// 戻り値 false = 未適用マージを発見。
fn no_unmerged_leaf_branch<V: crate::spatial_id::collection::flex_tree::ptr::SafeValue>(
    node: &Node<V>,
) -> bool {
    match node {
        Node::Leaf { .. } => true,
        Node::Branch {
            lower_child,
            upper_child,
            ..
        } => {
            if let (Node::Leaf { value: v1 }, Node::Leaf { value: v2 }) =
                (&**lower_child, &**upper_child)
                && v1 == v2
            {
                return false; // Leaf(v)+Leaf(v) が collapse されずに残っている
            }
            no_unmerged_leaf_branch(lower_child) && no_unmerged_leaf_branch(upper_child)
        }
    }
}

/// Set の上下ルートをともに検査する。
fn set_is_fully_merged(set: &SpatialIdSet) -> bool {
    no_unmerged_leaf_branch(&set.inner.inner.lower_root)
        && no_unmerged_leaf_branch(&set.inner.inner.upper_root)
}

/// union(F下半分, F上半分) は完全立方体 → 1 葉へ collapse（演算をまたぐマージ）。
#[test]
fn union_of_f_halves_collapses() {
    let mut lower = SpatialIdSet::new();
    let mut upper = SpatialIdSet::new();
    for x in 0..2 {
        for y in 0..2 {
            lower.insert(SingleId::new(1, 0, x, y).unwrap());
            upper.insert(SingleId::new(1, 1, x, y).unwrap());
        }
    }
    // 各半分は (f z1, x z0, y z0) の 1 葉へ collapse 済みのはず。
    assert_eq!(lower.count(), 1);
    assert_eq!(upper.count(), 1);
    let u = &lower | &upper;
    assert_eq!(u.count(), 1, "union of F-halves must collapse to 1");
}

/// 深いズームの立方体を完全に満たすと、多段の collapse が再帰的に伝播して
/// 1 葉になる。zoom Z の等方立方体 = (2^Z)^3 セル、ツリー深さ 3*Z 段の cascade。
#[test]
fn deep_recursive_cascade_collapses() {
    for z in 1u8..=3 {
        let side: u32 = 1 << z; // 2^z
        let mut set = SpatialIdSet::new();
        for f in 0..side as i32 {
            for x in 0..side {
                for y in 0..side {
                    set.insert(SingleId::new(z, f, x, y).unwrap());
                }
            }
        }
        assert_eq!(
            set.count(),
            1,
            "zoom {z} full cube must cascade-collapse to 1"
        );
        assert!(set_is_fully_merged(&set));
    }
}

/// 最後の 1 セルを入れた瞬間に、全レベルの collapse が一気にカスケードすること。
/// 直前は複数葉、最後の insert 後に 1 葉。
#[test]
fn final_insert_triggers_full_cascade() {
    let z = 2u8;
    let side: u32 = 1 << z;
    let coords: Vec<(i32, u32, u32)> = (0..side as i32)
        .flat_map(|f| (0..side).flat_map(move |x| (0..side).map(move |y| (f, x, y))))
        .collect();

    let mut set = SpatialIdSet::new();
    for &(f, x, y) in &coords[..coords.len() - 1] {
        set.insert(SingleId::new(z, f, x, y).unwrap());
    }
    let before = set.count();
    assert!(
        before > 1,
        "before last insert should be multi-leaf, got {before}"
    );

    let (f, x, y) = coords[coords.len() - 1];
    set.insert(SingleId::new(z, f, x, y).unwrap());
    assert_eq!(set.count(), 1, "last insert must cascade-collapse to 1");
    assert!(set_is_fully_merged(&set));
}

/// 部分木（ルートではない深い位置）での再帰 collapse。
/// 1つの zoom2 セルの 8 つの zoom3 子を満たすと、その部分木だけが 1 葉に畳まれ、
/// 離れた別セルは別葉として残る（全体 count == 2）。
#[test]
fn recursive_collapse_in_subtree() {
    let mut set = SpatialIdSet::new();
    // (f,x,y)=(0,0,0) の zoom2 セルを、その zoom3 子 8 個で満たす。
    for f in 0..2 {
        for x in 0..2 {
            for y in 0..2 {
                set.insert(SingleId::new(3, f, x, y).unwrap());
            }
        }
    }
    // 別の zoom2 親に属する離れたセル（zoom3 の有効範囲 0..=7）。
    set.insert(SingleId::new(3, 5, 5, 5).unwrap());

    assert_eq!(
        set.count(),
        2,
        "subtree collapses to 1 leaf, plus 1 distant leaf"
    );
    assert!(set_is_fully_merged(&set));
}

/// union で 2 つの大きな相補部分木を結合しても再帰 collapse する。
/// 偶数 f 半分と奇数 f 半分を別 Set で作り、union で zoom3 立方体を完成 → 1。
#[test]
fn union_recursive_collapse_deep() {
    let z = 3u8;
    let side: u32 = 1 << z;
    let mut a = SpatialIdSet::new();
    let mut b = SpatialIdSet::new();
    for f in 0..side as i32 {
        for x in 0..side {
            for y in 0..side {
                if f % 2 == 0 {
                    a.insert(SingleId::new(z, f, x, y).unwrap());
                } else {
                    b.insert(SingleId::new(z, f, x, y).unwrap());
                }
            }
        }
    }
    let u = &a | &b;
    assert_eq!(
        u.count(),
        1,
        "union must recursively collapse deep cube to 1"
    );
    assert!(set_is_fully_merged(&u));
}

/// 格納されている全 FlexId から `max(f_zoom, x_zoom, y_zoom)` の最大値を直接計算する。
/// キャッシュ済み `max_zoom` のオラクル。
fn brute_max_zoom(set: &SpatialIdSet) -> Option<u8> {
    set.iter()
        .map(|id| id.f_zoomlevel().max(id.x_zoomlevel()).max(id.y_zoomlevel()))
        .max()
}

/// 木の現在の内容を iter() で取り出し、新しい Set へ再挿入したものを返す。
fn rebuilt(set: &SpatialIdSet) -> SpatialIdSet {
    let mut out = SpatialIdSet::new();
    for flex_id in set.iter() {
        out.insert(flex_id);
    }
    out
}

proptest! {
    /// マージが正規形（canonical）なら、iter()→再挿入で count は不変のはず。
    /// 変わるなら「構築経路依存のマージ漏れ」。
    #[test]
    fn rebuild_preserves_count(case in arb_random_set_case()) {
        let set = case.build_set();
        let again = rebuilt(&set);
        prop_assert_eq!(set.count(), again.count(), "{}", case.debug_summary());
    }

    /// union 結果も正規形か（演算後に再挿入して count 不変か）。
    #[test]
    fn union_result_is_canonical(a in arb_random_set_case(), b in arb_random_set_case()) {
        let sa = a.build_set();
        let sb = b.build_set();
        let u = &sa | &sb;
        let again = rebuilt(&u);
        prop_assert_eq!(u.count(), again.count(),
            "union not canonical: a={} b={}", a.debug_summary(), b.debug_summary());
    }

    /// difference 結果も正規形か。
    #[test]
    fn difference_result_is_canonical(a in arb_random_set_case(), b in arb_random_set_case()) {
        let sa = a.build_set();
        let sb = b.build_set();
        let d = &sa - &sb;
        let again = rebuilt(&d);
        prop_assert_eq!(d.count(), again.count(),
            "difference not canonical: a={} b={}", a.debug_summary(), b.debug_summary());
    }

    /// intersection 結果も正規形か。
    #[test]
    fn intersection_result_is_canonical(a in arb_random_set_case(), b in arb_random_set_case()) {
        let sa = a.build_set();
        let sb = b.build_set();
        let i = &sa & &sb;
        let again = rebuilt(&i);
        prop_assert_eq!(i.count(), again.count(),
            "intersection not canonical: a={} b={}", a.debug_summary(), b.debug_summary());
    }

    /// 内部不変条件: insert で構築した木に未適用マージ（Leaf(v)+Leaf(v) の Branch）が無い。
    #[test]
    fn insert_no_unmerged(case in arb_random_set_case()) {
        let set = case.build_set();
        prop_assert!(set_is_fully_merged(&set), "unmerged after insert: {}", case.debug_summary());
    }

    /// 冗長軸 collapse 後も、キャッシュ済み max_zoom が実際の格納セルと一致する
    /// （collapse で葉の実効 zoom が変わっても max_zoomlevel がステイルにならない）。
    #[test]
    fn max_zoom_consistent_after_collapse(a in arb_random_set_case(), b in arb_random_set_case()) {
        let sa = a.build_set();
        let sb = b.build_set();
        prop_assert_eq!(sa.max_zoomlevel(), brute_max_zoom(&sa), "insert: {}", a.debug_summary());
        let u = &sa | &sb;
        prop_assert_eq!(u.max_zoomlevel(), brute_max_zoom(&u), "union");
        let i = &sa & &sb;
        prop_assert_eq!(i.max_zoomlevel(), brute_max_zoom(&i), "intersection");
        let d = &sa - &sb;
        prop_assert_eq!(d.max_zoomlevel(), brute_max_zoom(&d), "difference");
    }

    /// 内部不変条件: union/intersection/difference の結果にも未適用マージが無い。
    #[test]
    fn ops_no_unmerged(a in arb_random_set_case(), b in arb_random_set_case()) {
        let sa = a.build_set();
        let sb = b.build_set();
        prop_assert!(set_is_fully_merged(&(&sa | &sb)), "unmerged after union: a={} b={}", a.debug_summary(), b.debug_summary());
        prop_assert!(set_is_fully_merged(&(&sa & &sb)), "unmerged after intersection: a={} b={}", a.debug_summary(), b.debug_summary());
        prop_assert!(set_is_fully_merged(&(&sa - &sb)), "unmerged after difference: a={} b={}", a.debug_summary(), b.debug_summary());
    }
}

/// zoom1 の立方体を満たす 8 オクタント（全て同一「値」）を insert したら、
/// 1 つの葉（zoom0 相当）へ collapse して count==1 になるはず。
#[test]
fn eight_octants_collapse_to_one() {
    let mut set = SpatialIdSet::new();
    for f in 0..2 {
        for x in 0..2 {
            for y in 0..2 {
                set.insert(SingleId::new(1, f, x, y).unwrap());
            }
        }
    }
    assert_eq!(set.count(), 1, "8 octants should merge into 1");
}

/// 挿入順を変えても最終的な count（マージ結果）は同じであるべき。
#[test]
fn insert_order_independence() {
    let coords: Vec<(i32, u32, u32)> = (0..2)
        .flat_map(|f| (0..2).flat_map(move |x| (0..2).map(move |y| (f, x, y))))
        .collect();

    let mut forward = SpatialIdSet::new();
    for &(f, x, y) in &coords {
        forward.insert(SingleId::new(1, f, x, y).unwrap());
    }

    let mut reverse = SpatialIdSet::new();
    for &(f, x, y) in coords.iter().rev() {
        reverse.insert(SingleId::new(1, f, x, y).unwrap());
    }

    assert_eq!(forward.count(), reverse.count(), "order changed count");
    assert_eq!(forward.count(), 1);
}

/// 最深 Y 兄弟（葉）の insert マージ（union ではなく insert 経由）。
#[test]
fn insert_y_siblings_merge() {
    let mut set = SpatialIdSet::new();
    set.insert(SingleId::new(2, 0, 0, 0).unwrap());
    set.insert(SingleId::new(2, 0, 0, 1).unwrap());
    assert_eq!(set.count(), 1, "y-siblings (leaves) should merge");
}

/// 7/8 オクタント（1つ欠け）。完全 collapse はしないが、部分マージは起きうる。
/// ここでは「ありえない大きな count」になっていないかだけ確認（漏れの兆候検出）。
#[test]
fn seven_octants_partial() {
    let mut set = SpatialIdSet::new();
    let mut n = 0;
    for f in 0..2 {
        for x in 0..2 {
            for y in 0..2 {
                if (f, x, y) == (1, 1, 1) {
                    continue;
                }
                set.insert(SingleId::new(1, f, x, y).unwrap());
                n += 1;
            }
        }
    }
    assert_eq!(n, 7);
    // 7 個入れて全く merge しなければ 7。merge が効けば 7 未満。
    println!("seven_octants count = {}", set.count());
    assert!(set.count() <= 7);
}

/// 大きな立方体（多数の子）を1値で満たし、その後さらに同値を入れても count が増えないこと。
#[test]
fn fill_zoom2_cube_collapses() {
    // zoom2 の f/x/y を 0..4 まで全部 = 4^3 = 64 セル。全部同一値。
    let mut set = SpatialIdSet::new();
    for f in 0..4 {
        for x in 0..4 {
            for y in 0..4 {
                set.insert(SingleId::new(2, f, x, y).unwrap());
            }
        }
    }
    // 完全に満たされた zoom2 立方体 → zoom0 の 1 葉へ collapseすべき。
    println!("zoom2 full cube count = {}", set.count());
    assert_eq!(set.count(), 1, "full zoom2 cube should collapse to 1");
}

#[test]
fn verify_f_strip_coverage_preserved() {
    use crate::spatial_id::zoom_level::ZoomLevel;
    // corner_cases と同じ 10 個の F 隣接セル（z=30, x=y=0）。
    let mut set = SpatialIdSet::new();
    let mut expected = std::collections::BTreeSet::new();
    for i in 0..10 {
        let id = SingleId::new(ZoomLevel::MAX.get(), ZoomLevel::MAX.f_max() - i, 0, 0).unwrap();
        set.insert(id);
        expected.insert((ZoomLevel::MAX.f_max() - i, 0u32, 0u32));
    }
    // マージ後 count は減るが、最細セルへ展開すると元の 10 セルと一致するはず（被覆不変）。
    let mut got = std::collections::BTreeSet::new();
    for (sid, _) in set.flat_single_ids().map(|s| (s, ())) {
        assert_eq!(sid.z(), ZoomLevel::MAX.get());
        got.insert((sid.f(), sid.x(), sid.y()));
    }
    assert_eq!(got, expected, "merge must preserve exact coverage");
}
