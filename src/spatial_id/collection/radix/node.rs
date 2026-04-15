use crate::{Dimension, FlexId, Side};

#[derive(Debug, PartialEq, Clone)]
pub enum KDNode {
    Branch {
        axis: Dimension,
        lower_child: Option<Box<KDNode>>,
        upper_child: Option<Box<KDNode>>,
    },
    Leaf,
}

impl KDNode {
    /// ツリーに FlexId を挿入する
    pub fn insert(&mut self, target: FlexId, passed_f_z: u8, passed_x_z: u8, passed_y_z: u8) {
        // 1. 終了判定: ターゲットがこのノードの領域全体を覆い尽くしている場合
        if passed_f_z >= target.f_zoomlevel()
            && passed_x_z >= target.x_zoomlevel()
            && passed_y_z >= target.y_zoomlevel()
        {
            // これより下の細かい枝（既にあるものも含む）をすべて刈り取り、
            // 「ここは完全に埋まった」という1つの大きなLeafにする
            *self = KDNode::Leaf;
            return;
        }

        match self {
            KDNode::Branch {
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

                // ==========================================
                // 【修正の核心】
                // ターゲットがこの次元をすでに「カバー」している場合、
                // 空間の両側（LowerとUpper）にまたがっているため、両方に挿入する！
                // ==========================================
                if passed_z >= target_z {
                    Self::insert_into_child(lower_child, *axis, target.clone(), nf, nx, ny);
                    Self::insert_into_child(upper_child, *axis, target, nf, nx, ny);
                } else {
                    // まだカバーしていないので、対象となる片方の道にだけ進む
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

                // ボトムアップの自動結合（マージ）
                let can_merge = match (lower_child.as_deref(), upper_child.as_deref()) {
                    (Some(KDNode::Leaf), Some(KDNode::Leaf)) => true,
                    _ => false,
                };

                if can_merge {
                    *self = KDNode::Leaf;
                }
            }
            KDNode::Leaf => {
                // 【行き止まりバグの真実】
                // 既存の Leaf にぶつかったということは、「この空間は既にカバー済み」ということ。
                // したがって、これ以上細かい（または同じ）範囲を追加しても意味がないため、
                // ここで「何もしない（捨てる）」のが数学的に正しい挙動です。
                return;
            }
        }
    }

    /// 子ノードへの挿入処理をカプセル化したヘルパー関数
    fn insert_into_child(
        child_opt: &mut Option<Box<KDNode>>,
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
            None => {
                // 道がない場合は next_dimension で次の軸を決定して木を伸ばす
                match Self::next_dimension(current_axis, &target, nf, nx, ny) {
                    Some(next_axis) => {
                        let mut new_branch = KDNode::Branch {
                            axis: next_axis,
                            lower_child: None,
                            upper_child: None,
                        };
                        new_branch.insert(target, nf, nx, ny);
                        *child_opt = Some(Box::new(new_branch));
                    }
                    None => {
                        *child_opt = Some(Box::new(KDNode::Leaf));
                    }
                }
            }
        }
    }

    pub fn forking(target: &FlexId, axis: &Dimension, passed_z: &u8) -> Side {
        let (target_z, index) = match axis {
            Dimension::F => (target.f_zoomlevel(), target.f_index() as u32),
            Dimension::X => (target.x_zoomlevel(), target.x_index()),
            Dimension::Y => (target.y_zoomlevel(), target.y_index()),
        };

        // 呼び出し元で passed_z >= target_z のケースを除外しているため、
        // ここでは絶対にアンダーフローやパニックは起きません。
        let shift = target_z - 1 - passed_z;
        let bit = (index >> shift) & 1;

        if bit == 0 { Side::Lower } else { Side::Upper }
    }

    /// もう通り過ぎた各次元のズームレベルが引数になっている
    /// 次にどこの次元に行くべきかを判断する
    pub fn next_dimension(
        current_axis: Dimension, // &self の代わりに Dimension を受け取る
        target: &FlexId,
        passed_f_z: u8,
        passed_x_z: u8,
        passed_y_z: u8,
    ) -> Option<Dimension> {
        let f_passed = passed_f_z >= target.f_zoomlevel();
        let x_passed = passed_x_z >= target.x_zoomlevel();
        let y_passed = passed_y_z >= target.y_zoomlevel();

        // 全ての次元がPassできていたらNoneを返す
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
            KDNode::Branch {
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
            KDNode::Leaf => {
                results.push(current_id);
            }
        }
    }
}
