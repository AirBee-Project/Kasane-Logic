//! `FlexTreeCore2`のカノニカル不変条件・挿入アルゴリズムのテスト。

use super::*;
use crate::SingleId;
use alloc::vec::Vec;

/// (Leaf数, Branch数)を数える。
fn count_nodes<V>(node: &Node<V>) -> (usize, usize) {
    match node {
        Node::Leaf { .. } => (1, 0),
        Node::Branch { lower, upper, .. } => {
            let (l1, b1) = count_nodes(lower);
            let (l2, b2) = count_nodes(upper);
            (l1 + l2, b1 + b2 + 1)
        }
    }
}

fn collect_leaves<V: Clone>(node: &Node<V>, out: &mut Vec<(FlexId, V)>) {
    match node {
        Node::Leaf { id, value } => out.push((*id, value.clone())),
        Node::Branch { lower, upper, .. } => {
            collect_leaves(lower, out);
            collect_leaves(upper, out);
        }
    }
}

/// `target`を包含するLeafの値を返す(木を歩くだけの単純な参照実装用ルックアップ)。
fn get<'a, V>(node: &'a Node<V>, target: &FlexId) -> Option<&'a V> {
    match node {
        Node::Leaf { id, value } => id.contains(target).then_some(value),
        Node::Branch {
            cell,
            level,
            lower,
            upper,
            ..
        } => {
            if !cell.contains(target) {
                return None;
            }
            match forking(target, *level) {
                Side::Lower => get(lower, target),
                Side::Upper => get(upper, target),
            }
        }
    }
}

/// N1'(必須分岐点) / N1''(値一様マージ) / キャッシュ整合を再帰的に検証する。
fn check_canonical<V: PartialEq>(node: &Node<V>) {
    if let Node::Branch {
        cell,
        level,
        lower,
        upper,
        leaf_count,
    } = node
    {
        let axis = axis_of(*level);
        let depth = depth_of(*level);
        assert_eq!(
            axis_zoom(cell, axis),
            depth,
            "cellのこの軸のズームがlevelの深さと一致しない"
        );

        let lower_region = lower.region();
        let upper_region = upper.region();
        assert!(cell.contains(&lower_region), "cellがlower子を包含しない");
        assert!(cell.contains(&upper_region), "cellがupper子を包含しない");
        assert!(
            axis_zoom(&lower_region, axis) > depth,
            "lower子がこの軸で実ビットを持たない"
        );
        assert!(
            axis_zoom(&upper_region, axis) > depth,
            "upper子がこの軸で実ビットを持たない"
        );
        assert_eq!(
            forking(&lower_region, *level),
            Side::Lower,
            "lower子の実際の側が食い違う"
        );
        assert_eq!(
            forking(&upper_region, *level),
            Side::Upper,
            "upper子の実際の側が食い違う"
        );
        assert_eq!(
            *leaf_count,
            lower.leaf_count() + upper.leaf_count(),
            "leaf_countキャッシュが不整合"
        );

        // 必須分岐点/値一様マージ: 畳めるのに畳んでいないBranchがあってはならない。
        if let (
            Node::Leaf {
                id: lo_id,
                value: lo_v,
            },
            Node::Leaf {
                id: up_id,
                value: up_v,
            },
        ) = (&**lower, &**upper)
        {
            let collapsible = lo_v == up_v
                && Some(*lo_id) == split_axis(cell, axis, Side::Lower)
                && Some(*up_id) == split_axis(cell, axis, Side::Upper);
            assert!(
                !collapsible,
                "畳めるのに畳んでいないBranchがある(N1'/N1''違反)"
            );
        }

        check_canonical(lower);
        check_canonical(upper);
    }
}

/// `SingleId`にちょうど対応する`FlexId`を得る(時間区間を持たないテスト専用の変換)。
fn flex_id_of(id: SingleId) -> FlexId {
    let mut iter = id.into_iter();
    let flex_id = iter
        .next()
        .expect("SingleIdは最低1つのFlexIdへ分解されるはず");
    assert!(
        iter.next().is_none(),
        "テストで使うSingleIdは1個のFlexIdに収まる前提"
    );
    flex_id
}

fn tree_of(pairs: &[(SingleId, u32)]) -> FlexTreeCore2<u32> {
    let mut tree = FlexTreeCore2::new();
    for (id, value) in pairs {
        tree.insert(id.clone(), *value);
    }
    tree
}

#[test]
fn single_insert_creates_a_bare_leaf() {
    let id = SingleId::new(20, 1, 12345, 6789).unwrap();
    let tree = tree_of(&[(id.clone(), 42)]);

    let node = tree.upper.as_deref().or(tree.lower.as_deref()).unwrap();
    let (leaves, branches) = count_nodes(node);
    assert_eq!((leaves, branches), (1, 0), "単独挿入で分岐が生まれている");
    assert_eq!(get(node, &flex_id_of(id)), Some(&42));
}

/// 互いに素な(包含関係の無い)深いzoomの点を複数挿入しても、Branch数はちょうど`n-1`
/// (n個の葉を持つ2分木の内部ノード数)に収まる。旧設計ではzoomに比例した単鎖
/// Branchが実体化していた箇所。
#[test]
fn disjoint_deep_inserts_stay_at_exactly_n_minus_one_branches() {
    let ids = [
        SingleId::new(25, 1, 1_000_000, 2_000_000).unwrap(),
        SingleId::new(25, 1, 30_000_000, 4_000_000).unwrap(),
        SingleId::new(28, 1, 5_000_000, 6_000_000).unwrap(),
        SingleId::new(20, 1, 900_000, 100_000).unwrap(),
    ];
    let pairs: Vec<(SingleId, u32)> = ids
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, id)| (id, i as u32))
        .collect();

    let tree = tree_of(&pairs);
    let node = tree.upper.as_deref().unwrap();
    let (leaves, branches) = count_nodes(node);
    assert_eq!(leaves, ids.len());
    assert_eq!(
        branches,
        ids.len() - 1,
        "互いに素なn個の葉に対してBranch数がn-1になっていない(単鎖が残っている疑い)"
    );

    check_canonical(node);

    for (i, id) in ids.iter().enumerate() {
        assert_eq!(get(node, &flex_id_of(id.clone())), Some(&(i as u32)));
    }
}

#[test]
fn insertion_order_does_not_affect_content() {
    let ids = [
        SingleId::new(22, 1, 100, 200).unwrap(),
        SingleId::new(22, 1, 3_000_000, 200).unwrap(),
        SingleId::new(15, 1, 3, 900).unwrap(),
        SingleId::new(10, -1, 3, 500).unwrap(),
    ];

    let forward: Vec<(SingleId, u32)> = ids
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, id)| (id, i as u32))
        .collect();
    let mut backward = forward.clone();
    backward.reverse();

    let tree_a = tree_of(&forward);
    let tree_b = tree_of(&backward);

    let mut leaves_a = Vec::new();
    let mut leaves_b = Vec::new();
    if let Some(n) = tree_a.upper.as_deref() {
        collect_leaves(n, &mut leaves_a);
    }
    if let Some(n) = tree_a.lower.as_deref() {
        collect_leaves(n, &mut leaves_a);
    }
    if let Some(n) = tree_b.upper.as_deref() {
        collect_leaves(n, &mut leaves_b);
    }
    if let Some(n) = tree_b.lower.as_deref() {
        collect_leaves(n, &mut leaves_b);
    }
    leaves_a.sort_by_key(|(id, _)| id.encode());
    leaves_b.sort_by_key(|(id, _)| id.encode());

    assert_eq!(leaves_a, leaves_b, "挿入順序で最終内容が変わっている");
}

/// ちょうど隣り合う(親を単純に2分した)Segment同士が同じ値なら、1つのLeafへ畳まれる。
#[test]
fn equal_valued_siblings_collapse_into_one_leaf() {
    let coarse = SingleId::new(10, 1, 500, 500).unwrap();
    let coarse_id = flex_id_of(coarse);
    let lower_half = coarse_id.split_y(Side::Lower).unwrap();
    let upper_half = coarse_id.split_y(Side::Upper).unwrap();

    let mut tree = FlexTreeCore2::new();
    tree.insert(lower_half, 7u32);
    tree.insert(upper_half, 7u32);

    let node = tree.upper.as_deref().unwrap();
    let (leaves, branches) = count_nodes(node);
    assert_eq!(
        (leaves, branches),
        (1, 0),
        "値の等しい兄弟が畳まれず残っている"
    );
    match node {
        Node::Leaf { id, value } => {
            assert_eq!(*id, coarse_id);
            assert_eq!(*value, 7);
        }
        Node::Branch { .. } => panic!("Leafへ畳まれていない"),
    }
}

/// より粗い領域の挿入は、内側にあった細かい値を完全に上書きする。
#[test]
fn coarser_insert_overwrites_nested_finer_ones() {
    let coarse = SingleId::new(5, 1, 3, 3).unwrap();
    let coarse_id = flex_id_of(coarse.clone());
    // coarse の内側にある、より細かいSegment。
    let inner = coarse_id
        .split_x(Side::Lower)
        .unwrap()
        .split_y(Side::Upper)
        .unwrap();

    let mut tree = FlexTreeCore2::new();
    tree.insert(inner, 1u32);
    tree.insert(coarse, 2u32);

    let node = tree.upper.as_deref().unwrap();
    let (leaves, branches) = count_nodes(node);
    assert_eq!(
        (leaves, branches),
        (1, 0),
        "粗い上書きの後にnestedだった細片が残っている"
    );
    assert_eq!(get(node, &coarse_id), Some(&2));
    assert_eq!(get(node, &inner), Some(&2));
}

/// 粗い領域の内側に細かい値を挿入すると、その部分だけ値が変わり、残りは元の値のまま
/// (`promote`のnarrow+再帰経路を通る)。
#[test]
fn finer_insert_carves_a_hole_in_a_coarser_one() {
    let coarse = SingleId::new(5, 1, 3, 3).unwrap();
    let coarse_id = flex_id_of(coarse.clone());
    let hole = coarse_id
        .split_x(Side::Lower)
        .unwrap()
        .split_y(Side::Upper)
        .unwrap()
        .split_x(Side::Lower)
        .unwrap();
    // hole とは異なる兄弟(coarse配下だが hole を含まない具体的な点)。
    let elsewhere = coarse_id
        .split_x(Side::Upper)
        .unwrap()
        .split_y(Side::Upper)
        .unwrap();

    let mut tree = FlexTreeCore2::new();
    tree.insert(coarse, 100u32);
    tree.insert(hole, 200u32);

    let node = tree.upper.as_deref().unwrap();
    check_canonical(node);

    assert_eq!(
        get(node, &hole),
        Some(&200),
        "穴の中の値が上書きされていない"
    );
    assert_eq!(
        get(node, &elsewhere),
        Some(&100),
        "穴の外側は元の値のままであるべき"
    );
}
