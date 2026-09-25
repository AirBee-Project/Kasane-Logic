use alloc::boxed::Box;
use alloc::vec::Vec;
use core::mem;

use crate::{
    FlexId, Side,
    spatial_id::{collection::flex_tree::core::node::Dimension, relative_flex_id::RelativeFlexId},
};

pub struct FlexTreeCore2<V> {
    upper_root: Node<V>,
    lower_root: Node<V>,
}

impl<V: Clone + PartialEq> Default for FlexTreeCore2<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: Clone + PartialEq> FlexTreeCore2<V> {
    /// 空の[FlexTreeCore2]を作成する
    pub fn new() -> Self {
        FlexTreeCore2 {
            upper_root: Node::Leaf(None),
            lower_root: Node::Leaf(None),
        }
    }

    /// [FlexId]と値を挿入する。
    /// 既に値がある場合には上書きされる。
    pub fn insert(&mut self, target: FlexId, value: V) {
        // 地球の北半球か南半球かを判定する（F=0 は上側に属する）
        let (root_node, root_flex_id) = if target.f_index().is_negative() {
            (&mut self.lower_root, FlexId::LOWER_MAX)
        } else {
            (&mut self.upper_root, FlexId::UPPER_MAX)
        };

        // 当該のNodeに挿入する
        root_node.insert(root_flex_id, target, value);
    }

    /// 値を持つ全ての領域と値への参照を返す。
    pub fn iter(&self) -> impl Iterator<Item = (FlexId, &V)> {
        let mut out = Vec::new();
        self.upper_root.collect(FlexId::UPPER_MAX, &mut out);
        self.lower_root.collect(FlexId::LOWER_MAX, &mut out);
        out.into_iter()
    }
}

/// ノードは自分の領域を持たず、親から渡される領域 `this` の上で意味を持つ。
pub(crate) enum Node<V> {
    /// `this` 全体が同じ値（`None` は空）。
    Leaf(Option<V>),
    /// `this` を `dimension` 方向に二分する。
    Branch {
        dimension: Dimension,
        upper: Box<Node<V>>,
        lower: Box<Node<V>>,
    },
    /// `this` の内側の `path`（`this` を原点とした相対 ID）まで一気に降りる。`this` のうち `path` の外側は必ず空となる。`child` は空の Leaf にも Skip にもならない。
    Skip {
        path: Box<RelativeFlexId>,
        child: Box<Node<V>>,
    },
}

impl<V: Clone + PartialEq> Node<V> {
    pub fn insert(&mut self, this: FlexId, target: FlexId, value: V) {
        debug_assert!(this.contains(&target));

        match self {
            // 領域がちょうど一致したら、配下を丸ごと上書きする
            _ if this == target => *self = Node::Leaf(Some(value)),

            // 既に同じ値で埋まっているなら何もしない
            Node::Leaf(existing_value) if existing_value.as_ref() == Some(&value) => {}

            // 空なら、target まで一気に降りる
            Node::Leaf(None) => *self = Node::skip(Node::Leaf(Some(value)), target, this),

            // 値のある Leaf は周りにも値があるので段を飛ばせない。1段分割してから挿入し直す
            Node::Leaf(existing_value) => {
                let existing_value = existing_value.take();
                *self = Node::Branch {
                    dimension: next_split_dimension(&this, &target),
                    upper: Box::new(Node::Leaf(existing_value.clone())),
                    lower: Box::new(Node::Leaf(existing_value)),
                };
                self.insert(this, target, value);
            }

            // 子の領域と target の交差を、それぞれの子へ挿入する
            Node::Branch {
                dimension,
                upper,
                lower,
            } => {
                for (side, child) in [(Side::Upper, upper), (Side::Lower, lower)] {
                    let child_id = this.split_on(*dimension, side).unwrap();
                    if let Some(child_target) = target.intersection(&child_id) {
                        child.insert(child_id, child_target, value.clone());
                    }
                }
                self.merge_children(this);
            }

            Node::Skip { path, child } => {
                let region = path
                    .to_absolute(&this)
                    .expect("Skip の相対 ID は this から作ったので最大ズームを超えない");

                // Skip の先の領域内なら、降りて挿入する
                if region.contains(&target) {
                    child.insert(region, target, value);
                    let child = mem::replace(&mut **child, Node::Leaf(None));
                    *self = Node::embed(child, region, this);
                }
                // Skip の先を丸ごと覆うなら、target まで一気に降りる形に置き換える
                else if target.contains(&region) {
                    *self = Node::skip(Node::Leaf(Some(value)), target, this);
                }
                // はみ出すなら、両方を含む位置で二分してから挿入し直す
                else {
                    let child = mem::replace(&mut **child, Node::Leaf(None));
                    *self = Node::fork(child, region, &target, this);
                    self.insert(this, target, value);
                }
            }
        }
    }

    /// `this`の内側に`target`置く[Node::Skip]を作る。
    fn skip(child: Node<V>, target: FlexId, this: FlexId) -> Self {
        debug_assert!(!matches!(child, Node::Leaf(None) | Node::Skip { .. }));
        if target == this {
            return child;
        }
        Node::Skip {
            path: Box::new(
                target
                    .relative_to(&this)
                    .expect("this は target を包含する"),
            ),
            child: Box::new(child),
        }
    }

    /// 領域 `from` の上で意味を持つ `node` を、それを包含する領域 `to` の上へ置き直す。
    /// `to` のうち `from` の外側は空になる。
    fn embed(node: Node<V>, from: FlexId, to: FlexId) -> Self {
        match node {
            // 空はどこに置いても空
            Node::Leaf(None) => node,
            // Skip の先へさらに Skip しないよう、Skip の行き先から直接 to へ張り直す
            Node::Skip { path, child } => {
                let region = path
                    .to_absolute(&from)
                    .expect("Skip の相対 ID は from から作ったので最大ズームを超えない");
                Node::skip(*child, region, to)
            }
            node => Node::skip(node, from, to),
        }
    }

    /// `this` から `region` へ降りて `child` を置く Skip の代わりに、`region` と `target` の
    /// 共通の祖先で二分した Branch を作る。`region` 側に `child` を置き、反対側は空にする。
    fn fork(child: Node<V>, region: FlexId, target: &FlexId, this: FlexId) -> Self {
        let ancestor = region
            .common_ancestor(target)
            .expect("同じルート配下なので共通の祖先がある");
        let dimension = fork_dimension(&ancestor, &region, target);

        let upper_id = ancestor.split_on(dimension, Side::Upper).unwrap();
        let lower_id = ancestor.split_on(dimension, Side::Lower).unwrap();
        let (upper, lower) = if upper_id.contains(&region) {
            (Node::skip(child, region, upper_id), Node::Leaf(None))
        } else {
            (Node::Leaf(None), Node::skip(child, region, lower_id))
        };
        let branch = Node::Branch {
            dimension,
            upper: Box::new(upper),
            lower: Box::new(lower),
        };
        Node::skip(branch, ancestor, this)
    }

    /// Branch の子の書き換えで冗長になった分割を畳む。
    ///
    /// - 両方の子が同じ値（空を含む） → 全体がその値
    /// - 片方の子が空 → 残りの子を `this` の上へ段飛ばしで置き直す
    fn merge_children(&mut self, this: FlexId) {
        let Node::Branch {
            dimension,
            upper,
            lower,
        } = self
        else {
            return;
        };
        let dimension = *dimension;

        let (kept, side) = match (upper.as_mut(), lower.as_mut()) {
            (Node::Leaf(u), Node::Leaf(l)) if u == l => (Node::Leaf(u.take()), None),
            (Node::Leaf(None), kept) => (mem::replace(kept, Node::Leaf(None)), Some(Side::Lower)),
            (kept, Node::Leaf(None)) => (mem::replace(kept, Node::Leaf(None)), Some(Side::Upper)),
            _ => return,
        };
        *self = match side {
            None => kept,
            Some(side) => Node::embed(kept, this.split_on(dimension, side).unwrap(), this),
        };
    }

    /// 領域 `this` のこのノード配下で、値を持つ領域を `out` へ集める。
    fn collect<'a>(&'a self, this: FlexId, out: &mut Vec<(FlexId, &'a V)>) {
        match self {
            Node::Leaf(None) => {}
            Node::Leaf(Some(value)) => out.push((this, value)),
            Node::Branch {
                dimension,
                upper,
                lower,
            } => {
                for (side, child) in [(Side::Upper, upper), (Side::Lower, lower)] {
                    child.collect(this.split_on(*dimension, side).unwrap(), out);
                }
            }
            Node::Skip { path, child } => {
                let region = path
                    .to_absolute(&this)
                    .expect("Skip の相対 ID は this から作ったので最大ズームを超えない");
                child.collect(region, out)
            }
        }
    }
}

/// `this`が`target`より粗い最初の軸をF→X→Y→Tの順番で探索して返す。
/// `this`が`target`を完全に含む状態で呼び出さないとPanicする。
fn next_split_dimension(this: &FlexId, target: &FlexId) -> Dimension {
    Dimension::ALL
        .into_iter()
        .find(|&d| this.zoomlevel_on(d) < target.zoomlevel_on(d))
        .expect("target は this に真に包含されている")
}

/// [`Node::fork`] で共通の祖先 `ancestor` を二分する軸を返す。
///
/// `region` と `target` が別々の子へ分かれる軸を優先する。無ければ `region` が片側に収まる最初の軸。
fn fork_dimension(ancestor: &FlexId, region: &FlexId, target: &FlexId) -> Dimension {
    Dimension::ALL
        .into_iter()
        .find(|&d| ancestor.zoomlevel_on(d) < region.zoomlevel_on(d).min(target.zoomlevel_on(d)))
        .unwrap_or_else(|| next_split_dimension(ancestor, region))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial_id::collection::flex_tree::core::FlexTreeCore;

    /// Leaf と Branch の数。Skip は位置の移動だけなので数えない。
    fn node_count<V>(node: &Node<V>) -> usize {
        match node {
            Node::Leaf(_) => 1,
            Node::Branch { upper, lower, .. } => 1 + node_count(upper) + node_count(lower),
            Node::Skip { child, .. } => node_count(child),
        }
    }

    /// 既存の [`FlexTreeCore`] と同じ内容になっているかを確かめる。
    /// FlexTreeCore は正規形を持つので、同じ内容なら `==` になる。
    fn assert_same_as_reference(tree: &FlexTreeCore2<u64>, reference: &FlexTreeCore<u64>) {
        let ours: FlexTreeCore<u64> = tree.iter().map(|(id, v)| (id, *v)).collect();
        assert_eq!(&ours, reference);
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

    /// 各軸のズームが `0..max_zoom` のランダムな FlexId を作る。
    fn random_id(next: &mut impl FnMut(u64) -> u64, max_zoom: u64) -> FlexId {
        let fz = next(max_zoom) as u8;
        let xz = next(max_zoom) as u8;
        let yz = next(max_zoom) as u8;
        let f = next(2 << fz) as i32 - (1 << fz);
        let x = next(1 << xz) as u32;
        let y = next(1 << yz) as u32;
        // 時間軸が無いビルドでは全時間（ズーム0）しか作れない
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

    /// Skip を足しても Node の大きさが変わらない（Skip の中身は Box の先）。
    #[test]
    fn node_size_is_not_inflated_by_skip() {
        #[allow(dead_code)]
        enum WithoutSkip<V> {
            Leaf(Option<V>),
            Branch {
                dimension: Dimension,
                upper: Box<WithoutSkip<V>>,
                lower: Box<WithoutSkip<V>>,
            },
        }
        assert_eq!(
            core::mem::size_of::<Node<u64>>(),
            core::mem::size_of::<WithoutSkip<u64>>()
        );
    }

    /// 相対 ID は、元の祖先から降りると元の領域へ戻る。
    #[test]
    fn relative_id_round_trip() {
        let mut next = rng(42);
        let mut checked = 0;
        while checked < 1000 {
            let region = random_id(&mut next, 30);
            // 別のランダムな ID との共通の祖先を、region の祖先として使う（F の上下が違えば無い）
            let Some(ancestor) = region.common_ancestor(&random_id(&mut next, 30)) else {
                continue;
            };

            let relative = region.relative_to(&ancestor).unwrap();
            assert_eq!(relative.to_absolute(&ancestor).unwrap(), region);
            checked += 1;
        }
    }

    #[test]
    fn f_zero_goes_to_upper_root() {
        let mut tree = FlexTreeCore2::new();
        tree.insert(FlexId::new(3, 0, 3, 0, 3, 0).unwrap(), 1u64);
        assert_eq!(tree.iter().count(), 1);
        assert!(matches!(tree.lower_root, Node::Leaf(None)));
    }

    #[test]
    fn insert_whole_root_becomes_single_leaf() {
        let mut tree = FlexTreeCore2::new();
        tree.insert(FlexId::UPPER_MAX, 7u64);
        assert!(matches!(tree.upper_root, Node::Leaf(Some(7))));
    }

    #[test]
    fn sibling_halves_with_same_value_merge() {
        let mut tree = FlexTreeCore2::new();
        tree.insert(FlexId::new(1, 0, 0, 0, 0, 0).unwrap(), 5u64);
        tree.insert(FlexId::new(1, 1, 0, 0, 0, 0).unwrap(), 5u64);
        assert!(matches!(tree.upper_root, Node::Leaf(Some(5))));
    }

    #[test]
    fn overwrite_inside_filled_leaf_splits() {
        let mut tree = FlexTreeCore2::new();
        tree.insert(FlexId::UPPER_MAX, 1u64);
        tree.insert(FlexId::new(2, 1, 2, 3, 2, 0).unwrap(), 2u64);
        assert!(tree.iter().count() > 2);
        // 周りに値があるので段は飛ばさない
        assert!(matches!(tree.upper_root, Node::Branch { .. }));

        // 同じ場所を元の値で上書きすると、全体が1つの葉に戻る
        tree.insert(FlexId::new(2, 1, 2, 3, 2, 0).unwrap(), 1u64);
        assert!(matches!(tree.upper_root, Node::Leaf(Some(1))));
    }

    /// 空の木へ細かい点を入れても、一本道の Branch を作らず Skip 1 つで降りる。
    #[test]
    fn deep_point_in_empty_tree_skips_levels() {
        let mut tree = FlexTreeCore2::new();
        tree.insert(FlexId::new(20, 12345, 20, 54321, 20, 999).unwrap(), 1u64);
        assert!(
            matches!(&tree.upper_root, Node::Skip { child, .. } if matches!(**child, Node::Leaf(Some(1))))
        );
    }

    /// 離れた2点は、共通の祖先で分岐する Branch 1 つの下に並ぶ。
    #[test]
    fn two_distant_points_share_one_fork() {
        let mut tree = FlexTreeCore2::new();
        tree.insert(FlexId::new(20, 1, 20, 1, 20, 1).unwrap(), 1u64);
        tree.insert(FlexId::new(20, 1, 20, 900_000, 20, 1).unwrap(), 2u64);
        // 分岐の Branch 1つと、各点の葉
        assert_eq!(node_count(&tree.upper_root), 3);
        assert_eq!(tree.iter().count(), 2);
    }

    /// 乱数で上書き挿入を繰り返し、既存の FlexTreeCore と内容が一致することを確かめる。
    /// 粗い ID が重なり合う場合と、細かい ID が疎に散る場合（段飛ばしが多い）の両方を試す。
    #[test]
    fn random_inserts_match_flex_tree_core() {
        let mut next = rng(0x1234_5678_9abc_def0);
        for max_zoom in [5, 20] {
            for _ in 0..50 {
                let mut tree = FlexTreeCore2::new();
                let mut reference = FlexTreeCore::new();

                for _ in 0..40 {
                    let id = random_id(&mut next, max_zoom);
                    let value = next(3);

                    tree.insert(id, value);
                    reference.insert(core::iter::once(id), value);
                    assert_same_as_reference(&tree, &reference);
                }
            }
        }
    }
}
