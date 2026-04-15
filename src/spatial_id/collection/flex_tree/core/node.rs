use crate::{Dimension, FlexId, Side};

#[derive(PartialEq, Clone)]
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
    pub fn insert(
        &mut self,
        target: FlexId,
        value: V,
        passed_f_z: u8,
        passed_x_z: u8,
        passed_y_z: u8,
    ) {
        if passed_f_z >= target.f_zoomlevel()
            && passed_x_z >= target.x_zoomlevel()
            && passed_y_z >= target.y_zoomlevel()
        {
            *self = Node::Leaf { value };
            return;
        }

        match self {
            Node::Branch {
                axis,
                lower_child,
                upper_child,
            } => {
                let passed_z = match axis {
                    Dimension::F => passed_f_z,
                    Dimension::X => passed_x_z,
                    Dimension::Y => passed_y_z,
                };
                let target_z = match axis {
                    Dimension::F => target.f_zoomlevel(),
                    Dimension::X => target.x_zoomlevel(),
                    Dimension::Y => target.y_zoomlevel(),
                };

                let (nf, nx, ny) = match axis {
                    Dimension::F => (passed_f_z + 1, passed_x_z, passed_y_z),
                    Dimension::X => (passed_f_z, passed_x_z + 1, passed_y_z),
                    Dimension::Y => (passed_f_z, passed_x_z, passed_y_z + 1),
                };

                // ターゲットがこの次元をカバーしているなら両側に渡す
                if passed_z >= target_z {
                    Self::insert_into_child(
                        lower_child,
                        *axis,
                        target.clone(),
                        value.clone(),
                        nf,
                        nx,
                        ny,
                    );
                    Self::insert_into_child(upper_child, *axis, target, value, nf, nx, ny);
                } else {
                    let fork = Self::forking(&target, axis, &passed_z);
                    match fork {
                        Side::Lower => {
                            Self::insert_into_child(lower_child, *axis, target, value, nf, nx, ny)
                        }
                        Side::Upper => {
                            Self::insert_into_child(upper_child, *axis, target, value, nf, nx, ny)
                        }
                    }
                }

                let mut merge_value = None;
                if let (Some(Node::Leaf { value: v1 }), Some(Node::Leaf { value: v2 })) =
                    (lower_child.as_deref(), upper_child.as_deref())
                {
                    if v1 == v2 {
                        merge_value = Some(v1.clone());
                    }
                }

                if let Some(v) = merge_value {
                    *self = Node::Leaf { value: v };
                }
            }

            Node::Leaf {
                value: existing_value,
            } => {
                if *existing_value == value {
                    return;
                }

                let old_val = existing_value.clone();

                let start_axis = if passed_f_z < target.f_zoomlevel() {
                    Dimension::F
                } else if passed_x_z < target.x_zoomlevel() {
                    Dimension::X
                } else {
                    Dimension::Y
                };

                *self = Node::Branch {
                    axis: start_axis,
                    lower_child: Some(Box::new(Node::Leaf {
                        value: old_val.clone(),
                    })),
                    upper_child: Some(Box::new(Node::Leaf { value: old_val })),
                };

                self.insert(target, value, passed_f_z, passed_x_z, passed_y_z);
            }
        }
    }

    /// 子ノードへの挿入処理をカプセル化したヘルパー関数
    fn insert_into_child(
        child_opt: &mut Option<Box<Node<V>>>,
        current_axis: Dimension,
        target: FlexId,
        value: V,
        nf: u8,
        nx: u8,
        ny: u8,
    ) {
        match child_opt {
            Some(child) => {
                child.insert(target, value, nf, nx, ny);
            }
            None => match Self::next_dimension(current_axis, &target, nf, nx, ny) {
                Some(next_axis) => {
                    let mut new_branch = Node::Branch {
                        axis: next_axis,
                        lower_child: None,
                        upper_child: None,
                    };
                    new_branch.insert(target, value, nf, nx, ny);
                    *child_opt = Some(Box::new(new_branch));
                }
                None => {
                    // 全次元通過したら値付きのLeafを作る
                    *child_opt = Some(Box::new(Node::Leaf { value }));
                }
            },
        }
    }

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

    pub fn iter_leaves(&self, root_id: FlexId) -> super::convert::LeavesIter<'_, V> {
        super::convert::LeavesIter {
            stack: vec![(self, root_id)],
        }
    }
}
