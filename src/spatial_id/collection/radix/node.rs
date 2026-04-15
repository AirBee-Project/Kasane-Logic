use crate::{FlexId, Side};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Axis {
    F,
    X,
    Y,
}

#[derive(Debug, PartialEq, Clone)]
pub enum KDNode {
    Branch {
        axis: Axis,
        lower_child: Option<Box<KDNode>>,
        upper_child: Option<Box<KDNode>>,
    },
    Leaf,
}

impl KDNode {
    pub fn insert(&mut self, target: FlexId, passed_f_z: u8, passed_x_z: u8, passed_y_z: u8) {
        if passed_f_z >= target.f_zoomlevel()
            && passed_x_z >= target.x_zoomlevel()
            && passed_y_z >= target.y_zoomlevel()
        {
            // すでにこのノードがBranchだったとしても、Leafに上書きすることで
            // この範囲以下の細かい枝をすべて切り落とし（Pruning）、1つの大きなLeafに統合する
            *self = KDNode::Leaf;
            return;
        }

        match self {
            KDNode::Branch {
                axis,
                lower_child,
                upper_child,
            } => {
                // --- 既存のBranchロジック ---
                let passed_z = match axis {
                    Axis::F => passed_f_z,
                    Axis::X => passed_x_z,
                    Axis::Y => passed_y_z,
                };

                let fork = Self::forking(&target, axis, &passed_z);

                // 各次元のnext_zを計算
                let (nf, nx, ny) = match axis {
                    Axis::F => (passed_f_z + 1, passed_x_z, passed_y_z),
                    Axis::X => (passed_f_z, passed_x_z + 1, passed_y_z),
                    Axis::Y => (passed_f_z, passed_x_z, passed_y_z + 1),
                };

                let target_child = match fork {
                    Side::Lower => lower_child,
                    Side::Upper => upper_child,
                };

                match target_child {
                    Some(child) => child.insert(target, nf, nx, ny),
                    None => match Self::next_dimension(*axis, &target, nf, nx, ny) {
                        Some(next_axis) => {
                            let mut new_branch = KDNode::Branch {
                                axis: next_axis,
                                lower_child: None,
                                upper_child: None,
                            };
                            new_branch.insert(target, nf, nx, ny);
                            *target_child = Some(Box::new(new_branch));
                        }
                        None => {
                            *target_child = Some(Box::new(KDNode::Leaf));
                        }
                    },
                }
            }
            KDNode::Leaf => {
                // 【ここが修正ポイント：行き止まりバグの解消】
                // 冒頭の終了判定を抜けてきたということは、まだ深く潜る必要がある。
                // つまり、このLeafをBranchに分割（Split）しなければならない。

                // 次に進むべき軸を決定する（F -> X -> Y の優先順位で未達成のものを探す）
                let start_axis = if passed_f_z < target.f_zoomlevel() {
                    Axis::F
                } else if passed_x_z < target.x_zoomlevel() {
                    Axis::X
                } else {
                    Axis::Y
                };

                // 自分自身を Branch に作り変える
                *self = KDNode::Branch {
                    axis: start_axis,
                    lower_child: None,
                    upper_child: None,
                };

                // Branch になった自分自身に対して、再度 insert を呼び出す
                // 次の再帰では上の「KDNode::Branch」アームに入ることになる
                self.insert(target, passed_f_z, passed_x_z, passed_y_z);
            }
        }
    }

    pub fn forking(
        target: &FlexId,
        //現在の次元
        axis: &Axis,
        //現在の次元の通過済みのズームレベル
        passed_z: &u8,
    ) -> Side {
        match axis {
            Axis::F => {
                let target_z = target.f_zoomlevel();
                if *passed_z >= target_z {
                    panic!()
                }
                let shift = target_z - 1 - passed_z;
                let side = (target.f_index() as u32 >> shift) & 1;
                if side == 0 { Side::Lower } else { Side::Upper }
            }
            Axis::X => {
                let target_z = target.x_zoomlevel();
                if *passed_z >= target_z {
                    panic!()
                }
                if target_z == 0 {
                    return Side::Lower;
                }
                let shift = target_z - 1 - passed_z;
                let side = (target.x_index() >> shift) & 1;
                if side == 0 { Side::Lower } else { Side::Upper }
            }
            Axis::Y => {
                let target_z = target.y_zoomlevel();
                if *passed_z >= target_z {
                    panic!()
                }
                if target_z == 0 {
                    return Side::Lower;
                }
                let shift = target_z - 1 - passed_z;
                let side = (target.y_index() >> shift) & 1;
                if side == 0 { Side::Lower } else { Side::Upper }
            }
        }
    }

    /// もう通り過ぎた各次元のズームレベルが引数になっている
    /// 次にどこの次元に行くべきかを判断する
    pub fn next_dimension(
        current_axis: Axis, // &self の代わりに Axis を受け取る
        target: &FlexId,
        passed_f_z: u8,
        passed_x_z: u8,
        passed_y_z: u8,
    ) -> Option<Axis> {
        let f_passed = passed_f_z >= target.f_zoomlevel();
        let x_passed = passed_x_z >= target.x_zoomlevel();
        let y_passed = passed_y_z >= target.y_zoomlevel();

        // 全ての次元がPassできていたらNoneを返す
        if f_passed && x_passed && y_passed {
            return None;
        }

        match current_axis {
            Axis::F => {
                if !x_passed {
                    Some(Axis::X)
                } else if !y_passed {
                    Some(Axis::Y)
                } else {
                    Some(Axis::F)
                }
            }
            Axis::X => {
                if !y_passed {
                    Some(Axis::Y)
                } else if !f_passed {
                    Some(Axis::F)
                } else {
                    Some(Axis::X)
                }
            }
            Axis::Y => {
                if !f_passed {
                    Some(Axis::F)
                } else if !x_passed {
                    Some(Axis::X)
                } else {
                    Some(Axis::Y)
                }
            }
        }
    }

    pub fn collect_leaves(&self, results: &mut Vec<FlexId>, current_id: FlexId) {
        match self {
            KDNode::Branch {
                axis,
                lower_child,
                upper_child,
            } => {
                // 左に進む場合
                if let Some(child) = lower_child {
                    let next_id = match axis {
                        Axis::F => current_id.f_split(Side::Lower).unwrap(),
                        Axis::X => current_id.x_split(Side::Lower).unwrap(),
                        Axis::Y => current_id.y_split(Side::Lower).unwrap(),
                    };
                    child.collect_leaves(results, next_id);
                }
                // 右に進む場合
                if let Some(child) = upper_child {
                    let next_id = match axis {
                        Axis::F => current_id.f_split(Side::Upper).unwrap(),
                        Axis::X => current_id.x_split(Side::Upper).unwrap(),
                        Axis::Y => current_id.y_split(Side::Upper).unwrap(),
                    };
                    child.collect_leaves(results, next_id);
                }
            }
            KDNode::Leaf => {
                results.push(current_id);
            }
        }
    }
}
