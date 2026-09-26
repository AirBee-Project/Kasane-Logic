use alloc::format;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use super::FlexTreeCore2;
use super::node::Node;
use crate::{
    FlexId, Side,
    spatial_id::{dimension::Dimension, relative_flex_id::RelativeFlexId},
};

/// Leaf と Branch の数。Skip は位置の移動だけなので数えない。
fn node_count<V>(node: &Node<V>) -> usize {
    match node {
        Node::Leaf(_) => 1,
        Node::Branch { lower, upper, .. } => 1 + node_count(lower) + node_count(upper),
        Node::Skip { child, .. } => node_count(child),
    }
}

/// 領域 `this` のノードがカノニカル形の規則を守っているか検査する。
fn check_canonical<V: Clone + Ord + core::fmt::Debug>(
    node: &Node<V>,
    this: FlexId,
) -> Result<(), String> {
    match node {
        Node::Leaf(_) => Ok(()),
        Node::Branch {
            dimension,
            split_dimensions,
            value_range,
            lower,
            upper,
        } => {
            let expected = dimension.bit() | lower.split_dimensions() | upper.split_dimensions();
            if *split_dimensions != expected {
                return Err(format!("{this:?}: split_dimensions のキャッシュが古い"));
            }
            if lower.is_empty() || upper.is_empty() {
                return Err(format!("{this:?}: 片側が空の Branch（Skip にすべき）"));
            }
            if lower.split_dimensions() & dimension.bit() == 0 && lower == upper {
                return Err(format!("{this:?}: 割る必要のない Branch"));
            }
            if this.coarsest_dimension_in(*split_dimensions) != Some(*dimension) {
                return Err(format!("{this:?}: 一番粗い次元で割っていない"));
            }
            let (l_min, l_max) = lower
                .value_range()
                .ok_or_else(|| format!("{this:?}: lower が空の Branch"))?;
            let (u_min, u_max) = upper
                .value_range()
                .ok_or_else(|| format!("{this:?}: upper が空の Branch"))?;
            let expected_range = [l_min.min(u_min).clone(), l_max.max(u_max).clone()];
            if value_range != &expected_range {
                return Err(format!(
                    "{this:?}: value_range が不正: 実際 {value_range:?}, 期待 {expected_range:?}"
                ));
            }
            check_canonical(lower, this.split_on(*dimension, Side::Lower).unwrap())?;
            check_canonical(upper, this.split_on(*dimension, Side::Upper).unwrap())
        }
        Node::Skip {
            path,
            split_dimensions,
            child,
        } => {
            if matches!(**child, Node::Leaf(None) | Node::Skip { .. }) {
                return Err(format!("{this:?}: Skip の先が空または Skip"));
            }
            if *split_dimensions != (path.deeper_dimensions() | child.split_dimensions()) {
                return Err(format!("{this:?}: Skip の split_dimensions が不正"));
            }
            let region = path.to_absolute(&this).unwrap();
            if region == this {
                return Err(format!("{this:?}: 移動しない Skip"));
            }
            // 一本道の各段で、行き先が狭い次元から割っていること
            let mut at = this;
            while at != region {
                let finer = region.finer_dimensions_than(&at);
                let d = at
                    .coarsest_dimension_in(finer | child.split_dimensions())
                    .unwrap();
                if finer & d.bit() == 0 {
                    return Err(format!("{at:?}: Skip の途中で中身の次元を先に割るべき"));
                }
                at = at.split_toward(d, &region).unwrap();
            }
            check_canonical(child, region)
        }
    }
}

fn assert_canonical<V: Clone + Ord + core::fmt::Debug>(tree: &FlexTreeCore2<V>) {
    check_canonical(&tree.upper_root, FlexId::UPPER_MAX).unwrap();
    check_canonical(&tree.lower_root, FlexId::LOWER_MAX).unwrap();
}

/// 葉の列から木を組み立てる。
fn build<'a, V: Clone + Ord + 'a>(
    leaves: impl IntoIterator<Item = (FlexId, &'a V)>,
) -> FlexTreeCore2<V> {
    let mut tree = FlexTreeCore2::new();
    for (id, value) in leaves {
        tree.insert(id, value.clone());
    }
    tree
}

/// 木の中で `point` を含む葉の値。
fn value_at<V: Clone + Ord>(tree: &FlexTreeCore2<V>, point: &FlexId) -> Option<V> {
    tree.iter()
        .find(|(id, _)| id.contains(point))
        .map(|(_, v)| v.clone())
}

/// 参照モデル：書き込みの履歴から `point` の値を求める（後の書き込みが勝つ。`None` は削除）。
fn model_value_at<V: Clone>(writes: &[(FlexId, Option<V>)], point: &FlexId) -> Option<V> {
    writes
        .iter()
        .rev()
        .find(|(id, _)| id.contains(point))
        .and_then(|(_, v)| v.clone())
}

/// 決定的な擬似乱数（線形合同法）。`next(n)` は `0..n` を返す。
fn rng(mut seed: u64) -> impl FnMut(u64) -> u64 {
    move |n| {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (seed >> 33) % n
    }
}

/// 各次元のズームが `0..max_zoom` のランダムな FlexId を作る。
fn random_id(next: &mut impl FnMut(u64) -> u64, max_zoom: u64) -> FlexId {
    let fz = next(max_zoom) as u8;
    let xz = next(max_zoom) as u8;
    let yz = next(max_zoom) as u8;
    let f = next(2 << fz) as i32 - (1 << fz);
    let x = next(1 << xz) as u32;
    let y = next(1 << yz) as u32;
    // 時間次元が無いビルドでは全時間（ズーム0）しか作れない
    let tz = if cfg!(feature = "temporal_id") {
        next(max_zoom) as u8
    } else {
        0
    };
    let t = next(1 << tz);
    FlexId::new(fz, f, xz, x, yz, y)
        .unwrap()
        .with_time_segment(tz, t)
}

/// 標本点のズーム。[`random_id`] の `max_zoom` 以下の ID より必ず細かい。
const POINT_ZOOM: u8 = 20;

/// `region` の中のランダムな標本点（各次元のズーム [`POINT_ZOOM`]）。
fn random_point_in(next: &mut impl FnMut(u64) -> u64, region: &FlexId) -> FlexId {
    let mut deeper = |z: u8, i: i64| (i << (POINT_ZOOM - z)) + next(1 << (POINT_ZOOM - z)) as i64;
    let f = deeper(region.f_zoomlevel(), region.f_index() as i64);
    let x = deeper(region.x_zoomlevel(), region.x_index() as i64);
    let y = deeper(region.y_zoomlevel(), region.y_index() as i64);
    let (tz, t) = if cfg!(feature = "temporal_id") {
        (POINT_ZOOM, deeper(region.t_zoomlevel(), region.t() as i64))
    } else {
        (0, 0)
    };
    FlexId::new(
        POINT_ZOOM, f as i32, POINT_ZOOM, x as u32, POINT_ZOOM, y as u32,
    )
    .unwrap()
    .with_time_segment(tz, t as u64)
}

/// 書き込んだ各 ID の内側と、全体からランダムに選んだ標本点。
fn sample_points(next: &mut impl FnMut(u64) -> u64, ids: &[FlexId]) -> Vec<FlexId> {
    let mut points = Vec::new();
    for id in ids {
        for _ in 0..2 {
            points.push(random_point_in(next, id));
        }
    }
    for _ in 0..20 {
        let root = if next(2) == 0 {
            FlexId::UPPER_MAX
        } else {
            FlexId::LOWER_MAX
        };
        points.push(random_point_in(next, &root));
    }
    points
}

/// Node の大きさは、一番大きいバリアント（Branch か Skip）の中身にタグを足した大きさを超えない。
///
/// どちらが大きいかは値の型と時間次元の有無で変わる（`RelativeFlexId` は `temporal_id` ありで 24B、
/// なしで 16B）ため、大きさは決め打ちせず両方の中身から求める。
#[test]
fn node_size_is_bounded_by_largest_variant() {
    use core::mem::size_of;
    let branch = size_of::<(Dimension, u8, [u64; 2], Arc<()>, Arc<()>)>();
    let skip = size_of::<(RelativeFlexId, u8, Arc<()>)>();
    assert!(size_of::<Node<u64>>() <= branch.max(skip) + 8);
}

#[test]
fn f_zero_goes_to_upper_root() {
    let mut tree = FlexTreeCore2::new();
    tree.insert(FlexId::new(3, 0, 3, 0, 3, 0).unwrap(), 1u64);
    assert_eq!(tree.iter().count(), 1);
    assert!(tree.lower_root.is_empty());
}

#[test]
fn insert_whole_root_becomes_single_leaf() {
    let mut tree = FlexTreeCore2::new();
    tree.insert(FlexId::UPPER_MAX, 7u64);
    assert!(matches!(*tree.upper_root, Node::Leaf(Some(7))));
}

#[test]
fn sibling_halves_with_same_value_merge() {
    let mut tree = FlexTreeCore2::new();
    tree.insert(FlexId::new(1, 0, 0, 0, 0, 0).unwrap(), 5u64);
    tree.insert(FlexId::new(1, 1, 0, 0, 0, 0).unwrap(), 5u64);
    assert!(matches!(*tree.upper_root, Node::Leaf(Some(5))));
}

#[test]
fn overwrite_inside_filled_leaf_splits() {
    let mut tree = FlexTreeCore2::new();
    tree.insert(FlexId::UPPER_MAX, 1u64);
    tree.insert(FlexId::new(2, 1, 2, 3, 2, 0).unwrap(), 2u64);
    assert!(tree.iter().count() > 2);
    // 周りに値があるので段は飛ばさない
    assert!(matches!(*tree.upper_root, Node::Branch { .. }));

    // 同じ場所を元の値で上書きすると、全体が1つの葉に戻る
    tree.insert(FlexId::new(2, 1, 2, 3, 2, 0).unwrap(), 1u64);
    assert!(matches!(*tree.upper_root, Node::Leaf(Some(1))));
}

/// 空の木へ細かい点を入れても、一本道の Branch を作らず Skip 1 つで降りる。
#[test]
fn deep_point_in_empty_tree_skips_levels() {
    let mut tree = FlexTreeCore2::new();
    tree.insert(FlexId::new(20, 12345, 20, 54321, 20, 999).unwrap(), 1u64);
    assert!(
        matches!(&*tree.upper_root, Node::Skip { child, .. } if matches!(**child, Node::Leaf(Some(1))))
    );
}

/// 離れた2点は、分かれる地点の Branch 1 つの下に並ぶ。
#[test]
fn two_distant_points_share_one_fork() {
    let mut tree = FlexTreeCore2::new();
    tree.insert(FlexId::new(20, 1, 20, 1, 20, 1).unwrap(), 1u64);
    tree.insert(FlexId::new(20, 1, 20, 900_000, 20, 1).unwrap(), 2u64);
    // 分岐の Branch 1つと、各点の葉
    assert_eq!(node_count(&tree.upper_root), 3);
    assert_eq!(tree.iter().count(), 2);
    assert_canonical(&tree);
}

/// 挿入した点を消すと空の木に戻る。
#[test]
fn remove_restores_empty_tree() {
    let mut tree = FlexTreeCore2::new();
    let a = FlexId::new(20, 1, 20, 1, 20, 1).unwrap();
    let b = FlexId::new(20, 1, 20, 900_000, 20, 1).unwrap();
    tree.insert(a, 1u64);
    tree.insert(b, 2u64);
    tree.remove(a);
    tree.remove(b);
    assert_eq!(tree, FlexTreeCore2::new());
}

/// 乱数で上書き挿入と削除を繰り返し、参照モデルと値が一致し、常にカノニカル形であることを確かめる。
/// さらに、別の分け方の葉（各葉を半分に割ったもの・自分の葉を逆順）から組み直しても同じ形になることを確かめる。
/// 粗い ID が重なり合う場合と、細かい ID が疎に散る場合（段飛ばしが多い）の両方を試す。
#[test]
fn random_updates_match_model_and_stay_canonical() {
    let mut next = rng(0x1234_5678_9abc_def0);
    for max_zoom in [5, 20] {
        for _ in 0..50 {
            let mut tree = FlexTreeCore2::new();
            let mut writes: Vec<(FlexId, Option<u64>)> = Vec::new();

            for _ in 0..40 {
                let id = random_id(&mut next, max_zoom);
                if next(4) == 0 {
                    tree.remove(id);
                    writes.push((id, None));
                } else {
                    let value = next(3);
                    tree.insert(id, value);
                    writes.push((id, Some(value)));
                }
                assert_canonical(&tree);
            }

            let ids: Vec<FlexId> = writes.iter().map(|(id, _)| *id).collect();
            for point in sample_points(&mut next, &ids) {
                assert_eq!(value_at(&tree, &point), model_value_at(&writes, &point));
            }

            // 各葉を分割できる次元で半分に割った、別の分け方の葉から組み直す
            let halves: Vec<(FlexId, &u64)> = tree
                .iter()
                .flat_map(|(id, v)| {
                    match Dimension::ALL
                        .into_iter()
                        .find(|&d| id.split_on(d, Side::Upper).is_some())
                    {
                        Some(d) => [Side::Upper, Side::Lower]
                            .map(|side| (id.split_on(d, side).unwrap(), v))
                            .to_vec(),
                        None => alloc::vec![(id, v)],
                    }
                })
                .collect();
            assert_eq!(build(halves), tree);

            let mut own_leaves: Vec<(FlexId, &u64)> = tree.iter().collect();
            own_leaves.reverse();
            assert_eq!(build(own_leaves), tree);
        }
    }
}

/// 集合演算が参照モデルと一致し、結果もカノニカル形であることを確かめる。
#[test]
fn set_operations_match_model() {
    let mut next = rng(7);
    for max_zoom in [5, 20] {
        for _ in 0..50 {
            let (mut a, mut b) = (FlexTreeCore2::new(), FlexTreeCore2::new());
            let (mut ids_a, mut ids_b) = (Vec::new(), Vec::new());
            for _ in 0..20 {
                let id = random_id(&mut next, max_zoom);
                a.insert(id, ());
                ids_a.push(id);
                let id = random_id(&mut next, max_zoom);
                b.insert(id, ());
                ids_b.push(id);
            }

            let (union, intersection, difference) =
                (a.union(&b), a.intersection(&b), a.difference(&b));
            for tree in [&union, &intersection, &difference] {
                assert_canonical(tree);
            }

            let all_ids: Vec<FlexId> = ids_a.iter().chain(&ids_b).copied().collect();
            for point in sample_points(&mut next, &all_ids) {
                let in_a = ids_a.iter().any(|id| id.contains(&point));
                let in_b = ids_b.iter().any(|id| id.contains(&point));
                assert_eq!(value_at(&union, &point).is_some(), in_a || in_b);
                assert_eq!(value_at(&intersection, &point).is_some(), in_a && in_b);
                assert_eq!(value_at(&difference, &point).is_some(), in_a && !in_b);
            }
        }
    }
}

/// 木全体の value_range, min_value, max_value の基本動作テスト。
#[test]
fn value_range_basic() {
    let mut tree: FlexTreeCore2<u64> = FlexTreeCore2::new();
    assert_eq!(tree.value_range(), None);
    assert_eq!(tree.min_value(), None);
    assert_eq!(tree.max_value(), None);

    let id1 = FlexId::new(2, 0, 2, 0, 2, 0).unwrap();
    tree.insert(id1, 50);
    assert_eq!(tree.value_range(), Some((&50, &50)));
    assert_eq!(tree.min_value(), Some(&50));
    assert_eq!(tree.max_value(), Some(&50));

    let id2 = FlexId::new(2, 0, 2, 1, 2, 1).unwrap();
    tree.insert(id2, 20);
    assert_eq!(tree.value_range(), Some((&20, &50)));
    assert_eq!(tree.min_value(), Some(&20));
    assert_eq!(tree.max_value(), Some(&50));

    let id3 = FlexId::new(2, 0, 2, 2, 2, 2).unwrap();
    tree.insert(id3, 80);
    assert_eq!(tree.value_range(), Some((&20, &80)));
    assert_eq!(tree.min_value(), Some(&20));
    assert_eq!(tree.max_value(), Some(&80));
}

/// Branch の [min, max] を使った filter_range の枝刈りと参照モデル一致テスト。
#[test]
fn filter_range_prunes_and_matches_model() {
    let mut next = rng(42);
    for max_zoom in [4, 10] {
        for _ in 0..30 {
            let mut tree = FlexTreeCore2::new();
            let mut writes = Vec::new();
            let mut ids = Vec::new();

            for _ in 0..20 {
                let id = random_id(&mut next, max_zoom);
                let value = next(100) + 10; // 10..=109
                tree.insert(id, value);
                writes.push((id, Some(value)));
                ids.push(id);
            }
            assert_canonical(&tree);

            let (min_bound, max_bound) = (30u64, 70u64);
            let filtered = tree.filter_range(min_bound..=max_bound);
            assert_canonical(&filtered);

            // フィルター後のすべての葉の値が範囲内にあること
            for (_, val) in filtered.iter() {
                assert!(*val >= min_bound && *val <= max_bound);
            }

            // 標本点での値が参照モデルと完全に一致すること
            for point in sample_points(&mut next, &ids) {
                let expected =
                    model_value_at(&writes, &point).filter(|v| *v >= min_bound && *v <= max_bound);
                assert_eq!(value_at(&filtered, &point), expected);
            }

            // 全体を包含する範囲なら、木全体がそのまま返る（O(1) Pass）
            if let Some((&tree_min, &tree_max)) = tree.value_range() {
                let full = tree.filter_range(tree_min..=tree_max);
                assert_eq!(full, tree);
                // 非有界範囲 `..` でも同様
                let full_unbounded = tree.filter_range(..);
                assert_eq!(full_unbounded, tree);
            }

            // 完全に範囲外なら、空の木になる（O(1) Prune）
            let empty = tree.filter_range(1000..=2000);
            assert_eq!(empty.iter().count(), 0);
        }
    }
}
