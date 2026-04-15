use crate::{Dimension, FlexId, Side};

#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    Branch {
        axis: Dimension,
        lower_child: Option<Box<Node>>,
        upper_child: Option<Box<Node>>,
    },
    Leaf,
}

impl Node {
    pub fn insert(&mut self, target: FlexId, passed_f_z: u8, passed_x_z: u8, passed_y_z: u8) {
        if passed_f_z >= target.f_zoomlevel()
            && passed_x_z >= target.x_zoomlevel()
            && passed_y_z >= target.y_zoomlevel()
        {
            *self = Node::Leaf;
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

                // 各次元の次に渡すズームレベル
                let (nf, nx, ny) = match axis {
                    Dimension::F => (passed_f_z + 1, passed_x_z, passed_y_z),
                    Dimension::X => (passed_f_z, passed_x_z + 1, passed_y_z),
                    Dimension::Y => (passed_f_z, passed_x_z, passed_y_z + 1),
                };
                if passed_z >= target_z {
                    Self::insert_into_child(lower_child, *axis, target.clone(), nf, nx, ny);
                    Self::insert_into_child(upper_child, *axis, target, nf, nx, ny);
                } else {
                    let fork = Self::forking(&target, axis, &passed_z);
                    match fork {
                        Side::Lower => {
                            Self::insert_into_child(lower_child, *axis, target, nf, nx, ny)
                        }
                        Side::Upper => {
                            Self::insert_into_child(upper_child, *axis, target, nf, nx, ny)
                        }
                    }
                }
                let can_merge = match (lower_child.as_deref(), upper_child.as_deref()) {
                    (Some(Node::Leaf), Some(Node::Leaf)) => true,
                    _ => false,
                };

                if can_merge {
                    *self = Node::Leaf;
                }
            }
            Node::Leaf => {
                return;
            }
        }
    }

    fn insert_into_child(
        child_opt: &mut Option<Box<Node>>,
        current_axis: Dimension,
        target: FlexId,
        nf: u8,
        nx: u8,
        ny: u8,
    ) {
        match child_opt {
            Some(child) => {
                child.insert(target, nf, nx, ny);
            }
            None => match Self::next_dimension(current_axis, &target, nf, nx, ny) {
                Some(next_axis) => {
                    let mut new_branch = Node::Branch {
                        axis: next_axis,
                        lower_child: None,
                        upper_child: None,
                    };
                    new_branch.insert(target, nf, nx, ny);
                    *child_opt = Some(Box::new(new_branch));
                }
                None => {
                    *child_opt = Some(Box::new(Node::Leaf));
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

    pub fn collect_leaves(&self, results: &mut Vec<FlexId>, current_id: FlexId) {
        match self {
            Node::Branch {
                axis,
                lower_child,
                upper_child,
            } => {
                // 左に進む場合
                if let Some(child) = lower_child {
                    let next_id = match axis {
                        Dimension::F => current_id.f_split(Side::Lower).unwrap(),
                        Dimension::X => current_id.x_split(Side::Lower).unwrap(),
                        Dimension::Y => current_id.y_split(Side::Lower).unwrap(),
                    };
                    child.collect_leaves(results, next_id);
                }
                // 右に進む場合
                if let Some(child) = upper_child {
                    let next_id = match axis {
                        Dimension::F => current_id.f_split(Side::Upper).unwrap(),
                        Dimension::X => current_id.x_split(Side::Upper).unwrap(),
                        Dimension::Y => current_id.y_split(Side::Upper).unwrap(),
                    };
                    child.collect_leaves(results, next_id);
                }
            }
            Node::Leaf => {
                results.push(current_id);
            }
        }
    }
}
