use super::node::Node;
use super::ptr::SharedNode;

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

impl<V> Node<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue,
{
    pub fn union(
        a: &SharedNode<Self>,
        b: &SharedNode<Self>,
        current_level: u8,
        empty_leaf: &SharedNode<Node<V>>,
    ) -> SharedNode<Self> {
        if SharedNode::ptr_eq(a, b) {
            return a.clone();
        }

        if let Node::Leaf { value: Some(_) } = **a {
            return a.clone();
        }
        if let Node::Leaf { value: Some(_) } = **b {
            return b.clone();
        }
        if let Node::Leaf { value: None } = **a {
            return b.clone();
        }
        if let Node::Leaf { value: None } = **b {
            return a.clone();
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
                    || Self::union(al, bl, level + 1, empty_leaf),
                    || { Self::union(au, bu, level + 1, empty_leaf) }
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
                    || Self::union(al, b, level + 1, empty_leaf),
                    || Self::union(au, b, level + 1, empty_leaf)
                );
                return Self::compact_branch(level, new_lower, new_upper, a, b, empty_leaf);
            }
        } else {
            if let Node::Branch {
                lower_child: bl,
                upper_child: bu,
                ..
            } = &**b
            {
                let (new_lower, new_upper) = join_nodes!(
                    a.leaf_count() + b.leaf_count(),
                    || Self::union(a, bl, level + 1, empty_leaf),
                    || Self::union(a, bu, level + 1, empty_leaf)
                );
                return Self::compact_branch(level, new_lower, new_upper, a, b, empty_leaf);
            }
        }
        unreachable!();
    }

    pub fn intersection(
        a: &SharedNode<Self>,
        b: &SharedNode<Self>,
        current_level: u8,
        empty_leaf: &SharedNode<Node<V>>,
    ) -> SharedNode<Self> {
        if SharedNode::ptr_eq(a, b) {
            return a.clone();
        }

        if let Node::Leaf { value: None } = **a {
            return a.clone();
        }
        if let Node::Leaf { value: None } = **b {
            return b.clone();
        }
        if let Node::Leaf { value: Some(_) } = **a {
            return b.clone();
        }
        if let Node::Leaf { value: Some(_) } = **b {
            return a.clone();
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
                    || Self::intersection(al, bl, level + 1, empty_leaf),
                    || { Self::intersection(au, bu, level + 1, empty_leaf) }
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
                    || Self::intersection(al, b, level + 1, empty_leaf),
                    || { Self::intersection(au, b, level + 1, empty_leaf) }
                );
                return Self::compact_branch(level, new_lower, new_upper, a, b, empty_leaf);
            }
        } else {
            if let Node::Branch {
                lower_child: bl,
                upper_child: bu,
                ..
            } = &**b
            {
                let (new_lower, new_upper) = join_nodes!(
                    a.leaf_count() + b.leaf_count(),
                    || Self::intersection(a, bl, level + 1, empty_leaf),
                    || { Self::intersection(a, bu, level + 1, empty_leaf) }
                );
                return Self::compact_branch(level, new_lower, new_upper, a, b, empty_leaf);
            }
        }
        unreachable!();
    }

    pub fn difference(
        a: &SharedNode<Self>,
        b: &SharedNode<Self>,
        current_level: u8,
        empty_leaf: &SharedNode<Node<V>>,
    ) -> SharedNode<Self> {
        if SharedNode::ptr_eq(a, b) {
            return empty_leaf.clone();
        }

        if let Node::Leaf { value: None } = **a {
            return a.clone();
        }
        if let Node::Leaf { value: None } = **b {
            return a.clone();
        }
        if let Node::Leaf { value: Some(_) } = **b {
            return empty_leaf.clone();
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
                    || Self::difference(al, bl, level + 1, empty_leaf),
                    || { Self::difference(au, bu, level + 1, empty_leaf) }
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
                    || Self::difference(al, b, level + 1, empty_leaf),
                    || { Self::difference(au, b, level + 1, empty_leaf) }
                );
                return Self::compact_branch(level, new_lower, new_upper, a, b, empty_leaf);
            }
        } else {
            if let Node::Branch {
                lower_child: bl,
                upper_child: bu,
                ..
            } = &**b
            {
                let (new_lower, new_upper) = join_nodes!(
                    a.leaf_count() + b.leaf_count(),
                    || Self::difference(a, bl, level + 1, empty_leaf),
                    || { Self::difference(a, bu, level + 1, empty_leaf) }
                );
                return Self::compact_branch(level, new_lower, new_upper, a, b, empty_leaf);
            }
        }
        unreachable!();
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
            leaf_count: (new_lower.leaf_count() + new_upper.leaf_count()) as u32,
            max_zoom: Self::fold_max_zoom(level, &new_lower, &new_upper),
            lower_child: new_lower,
            upper_child: new_upper,
        })
    }
}
