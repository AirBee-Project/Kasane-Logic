use crate::{Dimension, FlexId, Side};
use std::rc::Rc;

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum Node<V>
where
    V: PartialEq + Clone,
{
    Branch {
        level: u8,
        leaf_count: usize,
        lower_child: Rc<Node<V>>,
        upper_child: Rc<Node<V>>,
    },
    Leaf {
        value: Option<V>,
    },
}

impl<V> Node<V>
where
    V: PartialEq + Clone,
{
    /// 各ノード以下の (値が Some の) Leaf の合計数を返す。O(1)で取得可能。
    pub fn leaf_count(&self) -> usize {
        match self {
            Node::Branch { leaf_count, .. } => *leaf_count,
            Node::Leaf { value: Some(_) } => 1,
            Node::Leaf { value: None } => 0,
        }
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
    fn completely_covers(target: &FlexId, level: u8) -> bool {
        let passed_f = level.div_ceil(3);
        let passed_x = (level + 1) / 3;
        let passed_y = level / 3;

        target.f_zoomlevel() <= passed_f
            && target.x_zoomlevel() <= passed_x
            && target.y_zoomlevel() <= passed_y
    }

    /// 持続的データ構造(Rc)に挿入し、必要に応じて新しいノードを生成して返す。
    pub fn insert(
        self: &Rc<Self>,
        target: &FlexId,
        value: &V,
        level: u8,
        empty_leaf: &Rc<Node<V>>,
    ) -> Rc<Self> {
        // 現在のノードがすでに Leaf であり、値が同一ならそのまま再利用(Result Reuse)
        if let Node::Leaf {
            value: Some(ref existing),
        } = **self
            && existing == value
        {
            return self.clone();
        }

        // 完全にターゲットが現在の空間全体を覆う場合、O(1)でLeafに置換する
        if Self::completely_covers(target, level) {
            return Rc::new(Node::Leaf {
                value: Some(value.clone()),
            });
        }

        let node_level = match **self {
            Node::Branch { level: l, .. } => l,
            Node::Leaf { .. } => 93, // 葉ノードの場合は仮想的に最大レベル (zoom 30)
        };

        let mut current_level = level;

        // Algorithm 1: Axis Skipping
        while current_level < node_level && Self::covers(target, current_level) {
            current_level += 1;
        }

        // 完全に target に覆い尽くされた場合、全体を塗りつぶす
        if current_level >= 93 {
            return Rc::new(Node::Leaf {
                value: Some(value.clone()),
            });
        }

        // 既存のツリーに欠けている階層 (Prepend missing level) を補うために Branch を作成する
        if current_level < node_level {
            let side = Self::forking(target, current_level);
            let (new_lower, new_upper) = match side {
                Side::Lower => {
                    let lo = self.insert(target, value, current_level + 1, empty_leaf);
                    (lo, self.clone())
                }
                Side::Upper => {
                    let hi = self.insert(target, value, current_level + 1, empty_leaf);
                    (self.clone(), hi)
                }
            };

            if let (Node::Leaf { value: v1 }, Node::Leaf { value: v2 }) = (&*new_lower, &*new_upper)
                && v1 == v2
            {
                if v1.is_none() {
                    return empty_leaf.clone();
                } else {
                    return Rc::new(Node::Leaf { value: v1.clone() });
                }
            }

            return Rc::new(Node::Branch {
                level: current_level,
                leaf_count: new_lower.leaf_count() + new_upper.leaf_count(),
                lower_child: new_lower,
                upper_child: new_upper,
            });
        }

        // current_level == node_level の場合
        match **self {
            Node::Branch {
                level: l,
                ref lower_child,
                ref upper_child,
                ..
            } => {
                let (new_lower, new_upper) = if Self::covers(target, l) {
                    // Target が現在の軸を完全に覆っている場合、両側の子へ挿入する
                    let lo = lower_child.insert(target, value, l + 1, empty_leaf);
                    let hi = upper_child.insert(target, value, l + 1, empty_leaf);
                    (lo, hi)
                } else {
                    let side = Self::forking(target, l);
                    match side {
                        Side::Lower => {
                            let lo = lower_child.insert(target, value, l + 1, empty_leaf);
                            (lo, upper_child.clone())
                        }
                        Side::Upper => {
                            let hi = upper_child.insert(target, value, l + 1, empty_leaf);
                            (lower_child.clone(), hi)
                        }
                    }
                };

                if let (Node::Leaf { value: v1 }, Node::Leaf { value: v2 }) =
                    (&*new_lower, &*new_upper)
                    && v1 == v2
                {
                    if v1.is_none() {
                        return empty_leaf.clone();
                    } else {
                        return Rc::new(Node::Leaf { value: v1.clone() });
                    }
                }

                // 子ポインタが変更されなかった場合、元の self を再利用(Result Reuse)
                if Rc::ptr_eq(&new_lower, lower_child) && Rc::ptr_eq(&new_upper, upper_child) {
                    return self.clone();
                }

                Rc::new(Node::Branch {
                    level: l,
                    leaf_count: new_lower.leaf_count() + new_upper.leaf_count(),
                    lower_child: new_lower,
                    upper_child: new_upper,
                })
            }
            Node::Leaf { .. } => unreachable!(),
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
