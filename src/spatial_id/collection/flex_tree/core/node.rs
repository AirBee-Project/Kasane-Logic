use crate::{Dimension, FlexId, Side};

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum Node<V>
where
    V: PartialEq + Clone,
{
    Branch {
        axis: Dimension,
        lower_child: Option<Box<Node<V>>>,
        upper_child: Option<Box<Node<V>>>,
    },
    Leaf {
        value: V,
    },
}

impl<V> Node<V>
where
    V: PartialEq + Clone,
{
    /// 指定された空間IDと値を現在ノード配下へ挿入し、葉ノード数の差分を返す。
    pub fn insert(&mut self, target: FlexId, value: V, passed: (u8, u8, u8)) -> isize {
        if Self::is_terminal_depth(&target, passed) {
            let delta = match self {
                Node::Leaf { value: existing } if existing == &value => 0,
                Node::Leaf { .. } => 0,
                Node::Branch { .. } => 1 - Self::subtree_leaf_count(self) as isize,
            };
            *self = Node::Leaf { value };
            return delta;
        }

        match self {
            Node::Branch {
                axis,
                lower_child,
                upper_child,
            } => {
                let mut delta = Self::insert_into_branch(
                    *axis,
                    lower_child,
                    upper_child,
                    target,
                    value,
                    passed,
                );

                if let Some(merged) = Self::mergeable_leaf_value(lower_child, upper_child) {
                    // 2つのLeafを1つに統合するため、葉ノード数差分は -1。
                    delta -= 1;
                    *self = Node::Leaf { value: merged };
                }

                delta
            }

            Node::Leaf {
                value: existing_value,
            } => {
                if *existing_value == value {
                    return 0;
                }

                let old_value = existing_value.clone();
                let start_axis = Self::start_axis_for_target(&target, passed);

                *self = Node::Branch {
                    axis: start_axis,
                    lower_child: Some(Box::new(Node::Leaf {
                        value: old_value.clone(),
                    })),
                    upper_child: Some(Box::new(Node::Leaf { value: old_value })),
                };

                // Leaf1個を子Leaf2個へ展開するので +1。
                let split_delta = 1;
                let recurse_delta = self.insert(target, value, passed);

                split_delta + recurse_delta
            }
        }
    }

    /// 現在の通過深度が target の全次元ズームに到達しているかを判定する。
    fn is_terminal_depth(target: &FlexId, passed: (u8, u8, u8)) -> bool {
        passed.0 >= target.f_zoomlevel()
            && passed.1 >= target.x_zoomlevel()
            && passed.2 >= target.y_zoomlevel()
    }

    /// Branchノードでの挿入方針を決定し、子ノード側で生じた葉ノード数差分を返す。
    fn insert_into_branch(
        axis: Dimension,
        lower_child: &mut Option<Box<Node<V>>>,
        upper_child: &mut Option<Box<Node<V>>>,
        target: FlexId,
        value: V,
        passed: (u8, u8, u8),
    ) -> isize {
        let passed_z = Self::passed_zoom(axis, passed.0, passed.1, passed.2);
        let target_z = Self::target_zoom(axis, &target);
        let next_passed = Self::next_passed_depth(axis, passed.0, passed.1, passed.2);

        let mut delta = 0;

        if passed_z >= target_z {
            delta += Self::insert_into_child(
                lower_child,
                axis,
                target.clone(),
                value.clone(),
                next_passed,
            );
            delta += Self::insert_into_child(upper_child, axis, target, value, next_passed);
        } else {
            match Self::forking(&target, &axis, &passed_z) {
                Side::Lower => {
                    delta += Self::insert_into_child(lower_child, axis, target, value, next_passed);
                }
                Side::Upper => {
                    delta += Self::insert_into_child(upper_child, axis, target, value, next_passed);
                }
            }
        }

        delta
    }

    /// 現在軸に対応する通過ズーム値を返す。
    fn passed_zoom(axis: Dimension, passed_f_z: u8, passed_x_z: u8, passed_y_z: u8) -> u8 {
        match axis {
            Dimension::F => passed_f_z,
            Dimension::X => passed_x_z,
            Dimension::Y => passed_y_z,
        }
    }

    /// 現在軸に対応する target 側のズーム値を返す。
    fn target_zoom(axis: Dimension, target: &FlexId) -> u8 {
        match axis {
            Dimension::F => target.f_zoomlevel(),
            Dimension::X => target.x_zoomlevel(),
            Dimension::Y => target.y_zoomlevel(),
        }
    }

    /// 現在軸を1段進めた次の通過ズーム状態を返す。
    fn next_passed_depth(
        axis: Dimension,
        passed_f_z: u8,
        passed_x_z: u8,
        passed_y_z: u8,
    ) -> (u8, u8, u8) {
        match axis {
            Dimension::F => (passed_f_z + 1, passed_x_z, passed_y_z),
            Dimension::X => (passed_f_z, passed_x_z + 1, passed_y_z),
            Dimension::Y => (passed_f_z, passed_x_z, passed_y_z + 1),
        }
    }

    /// 子が両方 Leaf かつ値一致なら、その統合値を返す。
    fn mergeable_leaf_value(
        lower_child: &Option<Box<Node<V>>>,
        upper_child: &Option<Box<Node<V>>>,
    ) -> Option<V> {
        if let (Some(Node::Leaf { value: v1 }), Some(Node::Leaf { value: v2 })) =
            (lower_child.as_deref(), upper_child.as_deref())
            && v1 == v2
        {
            return Some(v1.clone());
        }

        None
    }

    /// target を追跡するために次に展開を開始すべき軸を返す。
    fn start_axis_for_target(target: &FlexId, passed: (u8, u8, u8)) -> Dimension {
        if passed.0 < target.f_zoomlevel() {
            Dimension::F
        } else if passed.1 < target.x_zoomlevel() {
            Dimension::X
        } else {
            Dimension::Y
        }
    }

    /// 子ノードを生成または再帰利用し、葉ノード数差分を返しながら target を挿入する。
    fn insert_into_child(
        child_opt: &mut Option<Box<Node<V>>>,
        current_axis: Dimension,
        target: FlexId,
        value: V,
        passed: (u8, u8, u8),
    ) -> isize {
        match child_opt {
            Some(child) => child.insert(target, value, passed),
            None => match Self::next_dimension(current_axis, &target, passed.0, passed.1, passed.2)
            {
                Some(next_axis) => {
                    let mut new_branch = Node::Branch {
                        axis: next_axis,
                        lower_child: None,
                        upper_child: None,
                    };
                    let delta = new_branch.insert(target, value, passed);
                    *child_opt = Some(Box::new(new_branch));
                    delta
                }
                None => {
                    // 全次元通過したら値付きのLeafを作る
                    *child_opt = Some(Box::new(Node::Leaf { value }));
                    1
                }
            },
        }
    }

    /// 現在ノード配下の葉ノード数を再帰的に返す。
    fn subtree_leaf_count(node: &Node<V>) -> usize {
        match node {
            Node::Leaf { .. } => 1,
            Node::Branch {
                lower_child,
                upper_child,
                ..
            } => {
                let lower = lower_child
                    .as_deref()
                    .map(Self::subtree_leaf_count)
                    .unwrap_or(0);
                let upper = upper_child
                    .as_deref()
                    .map(Self::subtree_leaf_count)
                    .unwrap_or(0);
                lower + upper
            }
        }
    }

    /// 現在軸の分割ビットから、target が Lower/Upper のどちら側に進むかを返す。
    pub fn forking(target: &FlexId, axis: &Dimension, passed_z: &u8) -> Side {
        let (target_z, index) = match axis {
            Dimension::F => (target.f_zoomlevel(), target.f_index() as u32),
            Dimension::X => (target.x_zoomlevel(), target.x_index()),
            Dimension::Y => (target.y_zoomlevel(), target.y_index()),
        };

        let shift = target_z - 1 - passed_z;
        let bit = (index >> shift) & 1;

        if bit == 0 { Side::Lower } else { Side::Upper }
    }

    /// 現在軸と各次元の通過状況から、次に展開すべき軸を返す。
    pub fn next_dimension(
        current_axis: Dimension,
        target: &FlexId,
        passed_f_z: u8,
        passed_x_z: u8,
        passed_y_z: u8,
    ) -> Option<Dimension> {
        let f_passed = passed_f_z >= target.f_zoomlevel();
        let x_passed = passed_x_z >= target.x_zoomlevel();
        let y_passed = passed_y_z >= target.y_zoomlevel();

        if f_passed && x_passed && y_passed {
            return None;
        }

        match current_axis {
            Dimension::F => {
                if !x_passed {
                    Some(Dimension::X)
                } else if !y_passed {
                    Some(Dimension::Y)
                } else {
                    Some(Dimension::F)
                }
            }
            Dimension::X => {
                if !y_passed {
                    Some(Dimension::Y)
                } else if !f_passed {
                    Some(Dimension::F)
                } else {
                    Some(Dimension::X)
                }
            }
            Dimension::Y => {
                if !f_passed {
                    Some(Dimension::F)
                } else if !x_passed {
                    Some(Dimension::X)
                } else {
                    Some(Dimension::Y)
                }
            }
        }
    }
}
