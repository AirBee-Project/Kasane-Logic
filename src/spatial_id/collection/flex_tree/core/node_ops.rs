use super::node::Node;
use super::ptr::{SafeValue, SharedNode};

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
            let _ = $size;
            ($a(), $b())
        }
    }};
}

/// 2 つの木を突き合わせる二項演算の種類。
///
/// union / intersection / difference は「木を降りて子同士を突き合わせる」骨格が完全に
/// 共通で、**葉が絡む終端ケースだけ**が異なる。その差分を [`MergeOp::terminal`] に閉じ込め、
/// 降下ロジックは [`Node::merge`] の 1 本に統一する。
#[derive(Clone, Copy)]
pub(crate) enum MergeOp {
    Union,
    Intersection,
    Difference,
}

impl MergeOp {
    /// 葉が絡む終端ケースの結果を返す。`Some` なら再帰不要でそれが答え、`None` なら
    /// 両者を Branch とみなして降りる（difference で `a` が充填葉のときも `None` を返し、
    /// `a` を各子から引くために降下する）。
    #[inline]
    fn terminal<V: SafeValue>(
        self,
        a: &SharedNode<Node<V>>,
        b: &SharedNode<Node<V>>,
        empty_leaf: &SharedNode<Node<V>>,
    ) -> Option<SharedNode<Node<V>>> {
        use MergeOp::{Difference, Intersection, Union};

        // 構造共有された同一部分木は演算結果が自明（Result Reuse）。
        if SharedNode::ptr_eq(a, b) {
            return Some(match self {
                Union | Intersection => a.clone(),
                Difference => empty_leaf.clone(),
            });
        }

        let a_full = matches!(**a, Node::Leaf { value: Some(_) });
        let a_empty = matches!(**a, Node::Leaf { value: None });
        let b_full = matches!(**b, Node::Leaf { value: Some(_) });
        let b_empty = matches!(**b, Node::Leaf { value: None });

        match self {
            // a が充填 → a で覆う / b が充填 → b で覆う / 空側は相手を返す。
            Union if a_full => Some(a.clone()),
            Union if b_full => Some(b.clone()),
            Union if a_empty => Some(b.clone()),
            Union if b_empty => Some(a.clone()),

            // 空側で全体が空に / 充填側は相手をそのまま通す。
            Intersection if a_empty => Some(a.clone()),
            Intersection if b_empty => Some(b.clone()),
            Intersection if a_full => Some(b.clone()),
            Intersection if b_full => Some(a.clone()),

            // a が空 or b が空 → a のまま / b が充填 → 全消し。a が充填葉のときは
            // 下の None に落ちて降下し、b の各セルを引く。
            Difference if a_empty || b_empty => Some(a.clone()),
            Difference if b_full => Some(empty_leaf.clone()),

            _ => None,
        }
    }
}

impl<V> Node<V>
where
    V: SafeValue,
{
    /// 2 つの部分木 `a`・`b` を `op` で突き合わせて新しい部分木を返す（永続・構造共有）。
    ///
    /// レベルを揃えてから、両者が分岐する軸で子へ降りて再帰する。部分木が十分大きい間は
    /// `rayon::join` で左右を並列化する。結果は [`compact_branch`](Self::compact_branch) を
    /// 通して常に正規形になる。
    pub(crate) fn merge(
        a: &SharedNode<Self>,
        b: &SharedNode<Self>,
        op: MergeOp,
        current_level: u8,
        empty_leaf: &SharedNode<Node<V>>,
    ) -> SharedNode<Self> {
        if let Some(result) = op.terminal(a, b, empty_leaf) {
            return result;
        }

        // 終端でない = 少なくとも一方は Branch。両者が最初に分岐する軸までレベルを進める。
        let a_level = a.node_level();
        let b_level = b.node_level();
        let mut level = current_level;
        while level < a_level && level < b_level {
            level += 1;
        }

        #[cfg(feature = "rayon")]
        let size = a.leaf_count() + b.leaf_count();
        #[cfg(not(feature = "rayon"))]
        let size = 0;
        let (new_lower, new_upper) = if level == a_level && level == b_level {
            // 同じ軸で分岐: 下は下同士・上は上同士を突き合わせる。
            let (al, au) = a.children().unwrap();
            let (bl, bu) = b.children().unwrap();
            join_nodes!(
                size,
                || Self::merge(al, bl, op, level + 1, empty_leaf),
                || Self::merge(au, bu, op, level + 1, empty_leaf)
            )
        } else if level == a_level {
            // a だけが分岐: b（より粗い／充填葉）を a の両子へ配る。
            let (al, au) = a.children().unwrap();
            join_nodes!(
                size,
                || Self::merge(al, b, op, level + 1, empty_leaf),
                || Self::merge(au, b, op, level + 1, empty_leaf)
            )
        } else {
            // b だけが分岐: a を b の両子へ配る。
            let (bl, bu) = b.children().unwrap();
            join_nodes!(
                size,
                || Self::merge(a, bl, op, level + 1, empty_leaf),
                || Self::merge(a, bu, op, level + 1, empty_leaf)
            )
        };

        Self::compact_branch(level, new_lower, new_upper, a, b, empty_leaf)
    }

    /// 降下で作り直した子から Branch を組む。子が元の `a`/`b` と不変なら元ノードを共有する
    /// （Result Reuse）。正規形の入力なら再利用と [`mk`](Self::mk) の畳み込みは排他。
    #[inline]
    fn compact_branch(
        level: u8,
        new_lower: SharedNode<Node<V>>,
        new_upper: SharedNode<Node<V>>,
        a: &SharedNode<Node<V>>,
        b: &SharedNode<Node<V>>,
        empty_leaf: &SharedNode<Node<V>>,
    ) -> SharedNode<Self> {
        for src in [a, b] {
            if let Node::Branch {
                lower_child,
                upper_child,
                ..
            } = &**src
                && SharedNode::ptr_eq(&new_lower, lower_child)
                && SharedNode::ptr_eq(&new_upper, upper_child)
            {
                return src.clone();
            }
        }

        Self::mk(level, new_lower, new_upper, empty_leaf)
    }
}
