use super::ptr::SharedNode;
use crate::{Dimension, FlexId, Side};

#[derive(Debug, PartialEq, Clone, Eq)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "persist", rkyv(archive_bounds(V: 'static)))]
#[cfg_attr(
    feature = "persist",
    rkyv(serialize_bounds(
        __S: rkyv::ser::Writer + rkyv::ser::Allocator + rkyv::ser::Sharing,
        <__S as rkyv::rancor::Fallible>::Error: rkyv::rancor::Source,
    ))
)]
#[cfg_attr(
    feature = "persist",
    rkyv(deserialize_bounds(
        __D: rkyv::de::Pooling,
        <__D as rkyv::rancor::Fallible>::Error: rkyv::rancor::Source,
    ))
)]
#[cfg_attr(
    feature = "persist",
    rkyv(bytecheck(bounds(
        __C: rkyv::validation::ArchiveContext + rkyv::validation::SharedContext,
        <__C as rkyv::rancor::Fallible>::Error: rkyv::rancor::Source,
    )))
)]
pub enum Node<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue,
{
    Branch {
        level: u8,
        leaf_count: usize,
        max_zoom: u8,
        #[cfg_attr(feature = "persist", rkyv(omit_bounds))]
        lower_child: SharedNode<Node<V>>,
        #[cfg_attr(feature = "persist", rkyv(omit_bounds))]
        upper_child: SharedNode<Node<V>>,
    },
    Leaf {
        value: Option<V>,
    },
}

impl<V> Node<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue,
{
    /// 各ノード以下の (値が Some の) Leaf の合計数を返す。O(1)で取得可能。
    pub fn leaf_count(&self) -> usize {
        match self {
            Node::Branch { leaf_count, .. } => *leaf_count,
            Node::Leaf { value: Some(_) } => 1,
            Node::Leaf { value: None } => 0,
        }
    }

    /// このノードを `node_level`（自身の絶対ツリーレベル）に置いたときの、配下の値付き Leaf の
    /// FlexId ズームレベルの最大値を返す。Branch はキャッシュ済みの値を返すため O(1)。
    ///
    /// ツリーレベル `L` の Leaf が持つ FlexId のズームは `max(f, x, y) = ceil(L / 3)` に等しい
    /// （[`completely_covers`](Self::completely_covers) の式と整合）。値の無い Leaf は 0 を返す。
    pub(crate) fn max_zoom_at(&self, node_level: u8) -> u8 {
        match self {
            Node::Branch { max_zoom, .. } => *max_zoom,
            Node::Leaf { value: Some(_) } => node_level.div_ceil(3),
            Node::Leaf { value: None } => 0,
        }
    }

    /// レベル `level` の Branch を構築・更新する際の `max_zoom` を、両子の部分木から畳み上げる。
    /// 子はいずれもレベル `level + 1` に位置する。
    pub(crate) fn fold_max_zoom(level: u8, lower: &Node<V>, upper: &Node<V>) -> u8 {
        let child_level = level + 1;
        lower
            .max_zoom_at(child_level)
            .max(upper.max_zoom_at(child_level))
    }

    /// level から対象とする軸(F, X, Y) を返す
    pub fn axis(level: u8) -> Dimension {
        match level % 3 {
            0 => Dimension::F,
            1 => Dimension::X,
            2 => Dimension::Y,
            _ => unreachable!(),
        }
    }

    /// level から各軸の深度を返す
    pub fn depth(level: u8) -> u8 {
        level / 3
    }

    /// FlexId の指定次元に対するズームレベルを返す
    fn target_zoom(axis: Dimension, target: &FlexId) -> u8 {
        match axis {
            Dimension::F => target.f_zoomlevel(),
            Dimension::X => target.x_zoomlevel(),
            Dimension::Y => target.y_zoomlevel(),
        }
    }

    /// ターゲットAABB(FlexId)が現在の空間境界を特定の軸で「完全に覆う（covers）」か判定する。
    fn covers(target: &FlexId, level: u8) -> bool {
        let axis = Self::axis(level);
        let depth = Self::depth(level);
        Self::target_zoom(axis, target) <= depth
    }

    /// ターゲットAABB(FlexId)が現在の空間境界を全軸で完全に覆うか判定する。
    pub(crate) fn completely_covers_public(target: &FlexId, level: u8) -> bool {
        let passed_f = level.div_ceil(3);
        let passed_x = (level + 1) / 3;
        let passed_y = level / 3;

        target.f_zoomlevel() <= passed_f
            && target.x_zoomlevel() <= passed_x
            && target.y_zoomlevel() <= passed_y
    }

    /// 持続的データ構造に挿入します。
    /// 参照カウントが1の場合は `SharedNode::make_mut` を使用してインプレースで更新します (Copy-on-Write最適化)。
    pub fn insert_mut(
        node: &mut SharedNode<Self>,
        target: &FlexId,
        value: &V,
        level: u8,
        empty_leaf: &SharedNode<Node<V>>,
    ) {
        // 現在のノードがすでに Leaf であり、値が同一ならそのまま終了(Result Reuse)
        if let Node::Leaf {
            value: Some(ref existing),
        } = **node
            && existing == value
        {
            return;
        }

        // 完全にターゲットが現在の空間全体を覆う場合、O(1)でLeafに置換する
        if Self::completely_covers_public(target, level) {
            *node = SharedNode::new(Node::Leaf {
                value: Some(value.clone()),
            });
            return;
        }

        let node_level = match **node {
            Node::Branch { level: l, .. } => l,
            Node::Leaf { .. } => 93, // 葉ノードの場合は仮想的に最大レベル (zoom 30)
        };

        let mut current_level = level;

        // 無関係な次元をスキップ
        while current_level < node_level && Self::covers(target, current_level) {
            current_level += 1;
        }

        // 完全に target に覆い尽くされた場合、全体を塗りつぶす
        if current_level >= 93 {
            *node = SharedNode::new(Node::Leaf {
                value: Some(value.clone()),
            });
            return;
        }

        // 既存のツリーに欠けている階層 (Prepend missing level) を補うために Branch を作成する
        if current_level < node_level {
            let side = Self::forking(target, current_level);

            let (new_lower, new_upper) = match side {
                Side::Lower => {
                    let mut lo = node.clone();
                    Self::insert_mut(&mut lo, target, value, current_level + 1, empty_leaf);
                    (lo, node.clone())
                }
                Side::Upper => {
                    let mut hi = node.clone();
                    Self::insert_mut(&mut hi, target, value, current_level + 1, empty_leaf);
                    (node.clone(), hi)
                }
            };

            if let (Node::Leaf { value: v1 }, Node::Leaf { value: v2 }) = (&*new_lower, &*new_upper)
                && v1 == v2
            {
                if v1.is_none() {
                    *node = empty_leaf.clone();
                } else {
                    *node = SharedNode::new(Node::Leaf { value: v1.clone() });
                }
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

        // current_level == node_level の場合
        // ここが Copy-on-Write のコア。可能なら node の中身を mutable に取得する。
        let (should_merge, merged_val) = {
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
                    Self::insert_mut(lower_child, target, value, *l + 1, empty_leaf);
                    Self::insert_mut(upper_child, target, value, *l + 1, empty_leaf);
                } else {
                    match Self::forking(target, *l) {
                        Side::Lower => {
                            Self::insert_mut(lower_child, target, value, *l + 1, empty_leaf)
                        }
                        Side::Upper => {
                            Self::insert_mut(upper_child, target, value, *l + 1, empty_leaf)
                        }
                    }
                }

                *leaf_count = lower_child.leaf_count() + upper_child.leaf_count();
                *max_zoom = Self::fold_max_zoom(*l, lower_child, upper_child);

                if let (Node::Leaf { value: v1 }, Node::Leaf { value: v2 }) =
                    (&**lower_child, &**upper_child)
                {
                    if v1 == v2 {
                        (true, v1.clone())
                    } else {
                        (false, None)
                    }
                } else {
                    (false, None)
                }
            } else {
                unreachable!()
            }
        };

        if should_merge {
            if merged_val.is_none() {
                *node = empty_leaf.clone();
            } else {
                *node = SharedNode::new(Node::Leaf { value: merged_val });
            }
        }
    }

    /// target の次元ごとのインデックスビットを取得し、Lower / Upper を判定する。
    fn forking(target: &FlexId, level: u8) -> Side {
        let axis = Self::axis(level);
        let depth = Self::depth(level);

        let (target_z, index) = match axis {
            Dimension::F => (target.f_zoomlevel(), target.f_index() as u32),
            Dimension::X => (target.x_zoomlevel(), target.x_index()),
            Dimension::Y => (target.y_zoomlevel(), target.y_index()),
        };

        if depth >= target_z {
            return Side::Lower;
        }

        let shift = target_z - 1 - depth;
        let bit = (index >> shift) & 1;

        if bit == 0 { Side::Lower } else { Side::Upper }
    }
}
