use crate::FlexId;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Axis {
    F,
    X,
    Y,
}

///左右や上下を表す
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Bit {
    Zero,
    One,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    Branch {
        axis: Axis,
        zero_child: Option<Box<Node>>,
        one_child: Option<Box<Node>>,
    },
    Leaf {
        flex_id: FlexId,
    },
}

impl Node {
    /// ツリーに FlexId を挿入する
    /// passed_f_z, passed_x_z, passed_y_z: 現在のノードが評価している各次元の通過済みズームレベル
    pub fn insert(&mut self, target: FlexId, passed_f_z: u8, passed_x_z: u8, passed_y_z: u8) {
        // 安全対策: 既にターゲットの解像度に達しているのに Branch に来た場合は
        // 「既に細かい子ノードが存在している（重なっている）」ため、挿入をキャンセルする
        if passed_f_z >= target.f_zoomlevel()
            && passed_x_z >= target.x_zoomlevel()
            && passed_y_z >= target.y_zoomlevel()
        {
            *self = Node::Leaf { flex_id: target };
            return;
        }

        match self {
            Node::Branch {
                axis,
                zero_child, // Rust特有: 値を奪わず、ミュータブルな参照として受け取る
                one_child,
            } => {
                // 1. 現在の軸の通過済みズームレベルを取得
                let passed_z = match axis {
                    Axis::F => passed_f_z,
                    Axis::X => passed_x_z,
                    Axis::Y => passed_y_z,
                };

                // 2. ターゲットの分岐方向 (0 or 1) を決定
                let fork = Self::forking(&target, axis, &passed_z);

                // 3. 次の階層に渡すための、ズームレベルをインクリメントした値
                let next_f_z = if *axis == Axis::F {
                    passed_f_z + 1
                } else {
                    passed_f_z
                };
                let next_x_z = if *axis == Axis::X {
                    passed_x_z + 1
                } else {
                    passed_x_z
                };
                let next_y_z = if *axis == Axis::Y {
                    passed_y_z + 1
                } else {
                    passed_y_z
                };

                // 4. 進むべき子ノードの「書き換え可能な参照」を特定する (todo! の解消)
                let target_child = match fork {
                    Bit::Zero => zero_child,
                    Bit::One => one_child,
                };

                // 5. 道があるなら進み、無いなら作る
                match target_child {
                    Some(child) => {
                        // 【道がある場合】そのまま再帰的に深く潜る
                        child.insert(target, next_f_z, next_x_z, next_y_z);
                    }
                    None => {
                        // 【道がない場合】次にどの次元（Axis）に進むべきか判断
                        // ※判定には「次」のズームレベルを渡すのがポイント！
                        match Self::next_dimension(*axis, &target, next_f_z, next_x_z, next_y_z) {
                            Some(next_axis) => {
                                // まだ続きがあるなら新しい Branch を作る
                                let mut new_branch = Node::Branch {
                                    axis: next_axis,
                                    zero_child: None,
                                    one_child: None,
                                };
                                new_branch.insert(target, next_f_z, next_x_z, next_y_z);
                                *target_child = Some(Box::new(new_branch));
                            }
                            None => {
                                // 全ての次元を通過したなら、ここは終点（Leaf）になる
                                *target_child = Some(Box::new(Node::Leaf { flex_id: target }));
                            }
                        }
                    }
                }
            }
            Node::Leaf { flex_id: _ } => {
                // 仮にLeafに到達したならば、挿入するべき位置にはそれ以上のサイズの拡張空間IDがあるということなのでreturnしてOK
                return;
            }
        }
    }

    /// 拡張空間IDと次元とその次元の通過済みのズームレベルを渡すと、次のその次元における分岐方向を教えてくれる
    pub fn forking(target: &FlexId, axis: &Axis, passed_z: &u8) -> Bit {
        match axis {
            Axis::F => {
                let target_z = target.f_zoomlevel();
                if *passed_z >= target_z {
                    panic!()
                }
                let shift = target_z - 1 - passed_z;
                let bit = (target.f_index() as u32 >> shift) & 1;
                if bit == 0 { Bit::Zero } else { Bit::One }
            }
            Axis::X => {
                let target_z = target.x_zoomlevel();
                if *passed_z >= target_z {
                    panic!()
                }
                if target_z == 0 {
                    return Bit::Zero;
                }
                let shift = target_z - 1 - passed_z;
                let bit = (target.y_index() >> shift) & 1;
                if bit == 0 { Bit::Zero } else { Bit::One }
            }
            Axis::Y => {
                let target_z = target.y_zoomlevel();
                if *passed_z >= target_z {
                    panic!()
                }
                if target_z == 0 {
                    return Bit::Zero;
                }
                let shift = target_z - 1 - passed_z;
                let bit = (target.y_index() >> shift) & 1;
                if bit == 0 { Bit::Zero } else { Bit::One }
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

    /// 再帰的にツリーを巡回し、末端のLeaf（FlexId）を配列に収集する
    pub fn collect_leaves(&self, results: &mut Vec<FlexId>) {
        match self {
            Node::Branch {
                zero_child,
                one_child,
                ..
            } => {
                // 左(Zero)の枝があれば潜る
                if let Some(child) = zero_child {
                    child.collect_leaves(results);
                }
                // 右(One)の枝があれば潜る
                if let Some(child) = one_child {
                    child.collect_leaves(results);
                }
            }
            Node::Leaf { flex_id } => {
                // 葉に到達したら、結果配列に追加する（FlexIdはClone可能である必要があります）
                results.push(flex_id.clone());
            }
        }
    }
}
