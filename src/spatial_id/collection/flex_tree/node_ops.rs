use super::node::Node;
use super::ptr::SharedNode;
use crate::{FlexId, Side};

/// 部分木の合計葉数がこれ以上のときだけ `rayon::join` で分割する閾値。集合演算の再帰を全レベルで `join` するとタスク生成コストが並列化の利得を上回るため、大きな部分木（≒ 根に近い／密な領域）でだけ並列化し、小さくなったら逐次へ落とす。
#[cfg(feature = "rayon")]
pub(super) const PARALLEL_LEAF_CUTOFF: usize = 1024;

/// 部分木が十分大きいときだけ `rayon::join` で 2 分割し、小さいときは逐次に処理する。
macro_rules! join_nodes {
    ($size:expr, $a:expr, $b:expr) => {{
        #[cfg(feature = "rayon")]
        {
            if $size >= PARALLEL_LEAF_CUTOFF {
                rayon::join($a, $b)
            } else {
                ($a(), $b())
            }
        }
        #[cfg(not(feature = "rayon"))]
        {
            ($a(), $b())
        }
    }};
}

/// Node 集合演算の「値結合」を定義するトレイト（expr の `BinaryOperator` の Node 版）。
///
/// 空間の同一セルにおいて、両側/片側に値があるときの結合規則を与える。これにより
/// union/intersection/difference を1つの汎用 [`Node::combine`] で表現でき、値
/// （時間集合 [`TemporalSet`](crate::TemporalSet) など）を結合できる。
/// 3演算の制御フロー（レベルスキップ・枝再帰・compact）は完全に共通で、違いは
/// この結合規則だけである。
pub(crate) trait Combine<V>
where
    V: crate::spatial_id::collection::flex_tree::ptr::SafeValue,
{
    /// `b` 側が空葉のとき、`a` の部分木をそのまま結果にできるか。
    ///
    /// `true` にするには「すべての `v` について `a_only(v) == Some(v)`（恒等）」が
    /// 成立していなければならない（union / difference で真、intersection で偽）。
    /// これにより片側が空の部分木を O(1) で処理できる。
    const KEEP_A_WHEN_B_EMPTY: bool;
    /// `a` 側が空葉のとき、`b` の部分木をそのまま結果にできるか。
    ///
    /// `true` にするには「すべての `v` について `b_only(v) == Some(v)`（恒等）」が
    /// 成立していなければならない（union で真、intersection / difference で偽）。
    const KEEP_B_WHEN_A_EMPTY: bool;

    /// 両側に値があるとき。`None` を返すと不在。
    fn both(a: &V, b: &V) -> Option<V>;
    /// a のみ値があるとき。
    fn a_only(a: &V) -> Option<V>;
    /// b のみ値があるとき。
    fn b_only(b: &V) -> Option<V>;
    /// a と b が同一部分木（ptr_eq）のときの結果。
    /// union/intersection は a をそのまま、difference は空。
    fn on_identical(
        a: &SharedNode<Node<V>>,
        empty_leaf: &SharedNode<Node<V>>,
    ) -> SharedNode<Node<V>>;
}

impl<V> Node<V>
where
    V: crate::spatial_id::collection::flex_tree::ptr::SafeValue,
{
    /// 値結合 [`Combine`] を差し込んだ汎用の二項演算。
    ///
    /// 制御フローは [`union`](Self::union) と同一だが、唯一の base case
    /// 「両方が葉」で `C` の結合規則を適用する。葉 vs 枝は、葉を仮想レベル 93 として
    /// レベルロジックが枝の子へ降ろすことで自然に扱われる（＝葉の値を枝全体へ畳み込む）。
    pub(crate) fn combine<C: Combine<V>>(
        a: &SharedNode<Self>,
        b: &SharedNode<Self>,
        current_level: u8,
        empty_leaf: &SharedNode<Node<V>>,
    ) -> SharedNode<Self> {
        if SharedNode::ptr_eq(a, b) {
            return C::on_identical(a, empty_leaf);
        }

        // 片側が空葉なら部分木ごと O(1) で処理する（`Combine` の恒等/零元宣言に基づく）。
        // これにより疎な相手との結合が「重なる経路」だけのコストで済む。
        if let Node::Leaf { value: None } = **b {
            return if C::KEEP_A_WHEN_B_EMPTY {
                a.clone()
            } else {
                empty_leaf.clone()
            };
        }
        if let Node::Leaf { value: None } = **a {
            return if C::KEEP_B_WHEN_A_EMPTY {
                b.clone()
            } else {
                empty_leaf.clone()
            };
        }

        // 唯一の base case: 両方が葉 → 値を結合する。
        if let (Node::Leaf { value: av }, Node::Leaf { value: bv }) = (&**a, &**b) {
            let v = match (av, bv) {
                (Some(x), Some(y)) => C::both(x, y),
                (Some(x), None) => C::a_only(x),
                (None, Some(y)) => C::b_only(y),
                (None, None) => None,
            };
            return Self::leaf_of(v, empty_leaf);
        }

        let a_level = match **a {
            Node::Branch { level, .. } => level,
            Node::Leaf { .. } => 93,
        };
        let b_level = match **b {
            Node::Branch { level, .. } => level,
            Node::Leaf { .. } => 93,
        };

        let mut level = current_level;
        while level < a_level && level < b_level {
            level += 1;
        }

        if level == a_level && level == b_level {
            if let (
                Node::Branch {
                    lower_child: al,
                    upper_child: au,
                    ..
                },
                Node::Branch {
                    lower_child: bl,
                    upper_child: bu,
                    ..
                },
            ) = (&**a, &**b)
            {
                let (new_lower, new_upper) = join_nodes!(
                    a.leaf_count() + b.leaf_count(),
                    || Self::combine::<C>(al, bl, level + 1, empty_leaf),
                    || { Self::combine::<C>(au, bu, level + 1, empty_leaf) }
                );
                return Self::compact_branch(level, new_lower, new_upper, a, b, empty_leaf);
            }
        } else if level == a_level {
            if let Node::Branch {
                lower_child: al,
                upper_child: au,
                ..
            } = &**a
            {
                let (new_lower, new_upper) = join_nodes!(
                    a.leaf_count() + b.leaf_count(),
                    || Self::combine::<C>(al, b, level + 1, empty_leaf),
                    || { Self::combine::<C>(au, b, level + 1, empty_leaf) }
                );
                return Self::compact_branch(level, new_lower, new_upper, a, b, empty_leaf);
            }
        } else if let Node::Branch {
            lower_child: bl,
            upper_child: bu,
            ..
        } = &**b
        {
            let (new_lower, new_upper) = join_nodes!(
                a.leaf_count() + b.leaf_count(),
                || Self::combine::<C>(a, bl, level + 1, empty_leaf),
                || { Self::combine::<C>(a, bu, level + 1, empty_leaf) }
            );
            return Self::compact_branch(level, new_lower, new_upper, a, b, empty_leaf);
        }
        unreachable!();
    }

    /// 値から葉ノードを作る（`None` は共有 empty_leaf）。
    fn leaf_of(v: Option<V>, empty_leaf: &SharedNode<Node<V>>) -> SharedNode<Node<V>> {
        match v {
            None => empty_leaf.clone(),
            some => SharedNode::new(Node::Leaf { value: some }),
        }
    }

    #[inline]
    fn compact_branch(
        level: u8,
        new_lower: SharedNode<Node<V>>,
        new_upper: SharedNode<Node<V>>,
        a: &SharedNode<Node<V>>,
        b: &SharedNode<Node<V>>,
        empty_leaf: &SharedNode<Node<V>>,
    ) -> SharedNode<Self> {
        if let Some(rep) = Self::collapse_equal_children(&new_lower, &new_upper, level, empty_leaf)
        {
            return rep;
        }

        if let Node::Branch {
            lower_child: al,
            upper_child: au,
            ..
        } = &**a
            && SharedNode::ptr_eq(&new_lower, al)
            && SharedNode::ptr_eq(&new_upper, au)
        {
            return a.clone();
        }
        if let Node::Branch {
            lower_child: bl,
            upper_child: bu,
            ..
        } = &**b
            && SharedNode::ptr_eq(&new_lower, bl)
            && SharedNode::ptr_eq(&new_upper, bu)
        {
            return b.clone();
        }

        SharedNode::new(Node::Branch {
            level,
            leaf_count: new_lower.leaf_count() + new_upper.leaf_count(),
            max_zoom: Self::fold_max_zoom(level, &new_lower, &new_upper),
            lower_child: new_lower,
            upper_child: new_upper,
        })
    }

    /// 単一の空間セル `target` に値 `value` を、既存の葉値と [`Combine`] で**マージ**しながら
    /// インプレース挿入する（Copy-on-Write）。
    ///
    /// [`insert_mut`](Node::insert_mut) が葉値を**置換**するのに対し、こちらは重なる葉で
    /// `C::both` / 空セルで `C::b_only` を適用する。単一要素の `single` 木を作って
    /// [`combine`](Self::combine) する経路と結果は同一だが、使い捨て木の確保と木構造の
    /// 再構築を避けられる（有限時間挿入の定数倍を削減）。制御フロー（レベルスキップ・
    /// 欠損レベルの prepend・枝再帰・collapse）は `insert_mut` と同型。
    pub(crate) fn insert_combine_mut<C: Combine<V>>(
        node: &mut SharedNode<Self>,
        target: &FlexId,
        value: &V,
        level: u8,
        empty_leaf: &SharedNode<Node<V>>,
    ) {
        // target がこの node 領域全体を覆う → 配下すべてへ value を畳み込む。
        if Self::completely_covers_public(target, level) {
            Self::broadcast_combine::<C>(node, value, empty_leaf);
            return;
        }

        let node_level = match **node {
            Node::Branch { level: l, .. } => l,
            Node::Leaf { .. } => 93,
        };

        let mut current_level = level;
        while current_level < node_level && Self::covers(target, current_level) {
            current_level += 1;
        }

        if current_level >= 93 {
            Self::broadcast_combine::<C>(node, value, empty_leaf);
            return;
        }

        // 欠損レベルの prepend: 分岐を作り、既存 node を両子へ複製（＝target 外は既存値を保持）、
        // target 側だけ深く再帰してマージする。
        if current_level < node_level {
            let side = Self::forking(target, current_level);
            let (new_lower, new_upper) = match side {
                Side::Lower => {
                    let mut lo = node.clone();
                    Self::insert_combine_mut::<C>(
                        &mut lo,
                        target,
                        value,
                        current_level + 1,
                        empty_leaf,
                    );
                    (lo, node.clone())
                }
                Side::Upper => {
                    let mut hi = node.clone();
                    Self::insert_combine_mut::<C>(
                        &mut hi,
                        target,
                        value,
                        current_level + 1,
                        empty_leaf,
                    );
                    (node.clone(), hi)
                }
            };

            if let Some(rep) =
                Self::collapse_equal_children(&new_lower, &new_upper, current_level, empty_leaf)
            {
                *node = rep;
                return;
            }

            *node = SharedNode::new(Node::Branch {
                level: current_level,
                leaf_count: new_lower.leaf_count() + new_upper.leaf_count(),
                max_zoom: Self::fold_max_zoom(current_level, &new_lower, &new_upper),
                lower_child: new_lower,
                upper_child: new_upper,
            });
            return;
        }

        // current_level == node_level: 枝の該当子へ降りる（CoW）。
        let replacement = {
            let mut_node = SharedNode::make_mut(node);
            if let Node::Branch {
                level: l,
                lower_child,
                upper_child,
                leaf_count,
                max_zoom,
            } = mut_node
            {
                if Self::covers(target, *l) {
                    Self::insert_combine_mut::<C>(lower_child, target, value, *l + 1, empty_leaf);
                    Self::insert_combine_mut::<C>(upper_child, target, value, *l + 1, empty_leaf);
                } else {
                    match Self::forking(target, *l) {
                        Side::Lower => Self::insert_combine_mut::<C>(
                            lower_child,
                            target,
                            value,
                            *l + 1,
                            empty_leaf,
                        ),
                        Side::Upper => Self::insert_combine_mut::<C>(
                            upper_child,
                            target,
                            value,
                            *l + 1,
                            empty_leaf,
                        ),
                    }
                }

                *leaf_count = lower_child.leaf_count() + upper_child.leaf_count();
                *max_zoom = Self::fold_max_zoom(*l, lower_child, upper_child);

                Self::collapse_equal_children(lower_child, upper_child, *l, empty_leaf)
            } else {
                unreachable!()
            }
        };

        if let Some(rep) = replacement {
            *node = rep;
        }
    }

    /// この node 領域全体へ `value` を [`Combine`] で畳み込む（葉ならマージ、枝なら両子へ再帰）。
    fn broadcast_combine<C: Combine<V>>(
        node: &mut SharedNode<Self>,
        value: &V,
        empty_leaf: &SharedNode<Node<V>>,
    ) {
        match &**node {
            Node::Leaf { value: existing } => {
                let merged = match existing {
                    Some(a) => C::both(a, value),
                    None => C::b_only(value),
                };
                *node = match merged {
                    Some(v) => SharedNode::new(Node::Leaf { value: Some(v) }),
                    None => empty_leaf.clone(),
                };
            }
            Node::Branch { .. } => {
                let replacement = {
                    let mut_node = SharedNode::make_mut(node);
                    if let Node::Branch {
                        level: l,
                        lower_child,
                        upper_child,
                        leaf_count,
                        max_zoom,
                    } = mut_node
                    {
                        Self::broadcast_combine::<C>(lower_child, value, empty_leaf);
                        Self::broadcast_combine::<C>(upper_child, value, empty_leaf);
                        *leaf_count = lower_child.leaf_count() + upper_child.leaf_count();
                        *max_zoom = Self::fold_max_zoom(*l, lower_child, upper_child);
                        Self::collapse_equal_children(lower_child, upper_child, *l, empty_leaf)
                    } else {
                        unreachable!()
                    }
                };
                if let Some(rep) = replacement {
                    *node = rep;
                }
            }
        }
    }
}
