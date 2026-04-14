use crate::{
    FlexId,
    spatial_id::collection::radix::node::{Axis, Node},
};

pub mod node;
#[derive(Debug, PartialEq, Clone)]
pub struct VBitTree {
    pub root: Option<Box<Node>>,
}

impl VBitTree {
    /// 新しい空のツリーを作成する
    pub fn new() -> Self {
        Self { root: None }
    }

    /// ツリーに拡張空間IDを挿入します。
    /// （親ブロックによる子ブロックの上書き・吸収も自動で行われ、Setの制約が保たれます）
    pub fn insert(&mut self, target: FlexId) {
        // 1. ツリーが空 (None) の場合の初期化
        if self.root.is_none() {
            // もしターゲットが「世界全体（すべてのズームレベルが0）」なら、いきなりLeafにする
            if target.f_zoomlevel() == 0 && target.x_zoomlevel() == 0 && target.y_zoomlevel() == 0 {
                self.root = Some(Box::new(Node::Leaf { flex_id: target }));
                return;
            }

            // それ以外の場合は、最初の分岐（Branch）を作る
            // 基本はF軸からだが、非対称ボクセルでFのズームレベルが最初から0の場合はXやYから始める
            let start_axis = if target.f_zoomlevel() > 0 {
                Axis::F
            } else if target.x_zoomlevel() > 0 {
                Axis::X
            } else {
                Axis::Y
            };

            self.root = Some(Box::new(Node::Branch {
                axis: start_axis,
                zero_child: None,
                one_child: None,
            }));
        }

        // 2. ルートノードから再帰的に挿入処理を開始する
        // (スタート地点なので、通過済みのズームレベルはすべて 0 を渡す)
        if let Some(ref mut root_node) = self.root {
            root_node.insert(target, 0, 0, 0);
        }
    }

    /// ツリー内に存在するすべての拡張空間ID（重なりが排除されたSet）を出力します。
    pub fn output(&self) -> Vec<FlexId> {
        let mut results = Vec::new();

        // ツリーの中身が存在すれば、ノードを巡回してLeafを回収する
        if let Some(ref root_node) = self.root {
            root_node.collect_leaves(&mut results);
        }

        results
    }
}
